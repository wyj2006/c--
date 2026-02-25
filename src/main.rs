pub mod ast;
pub mod codegen;
pub mod ctype;
pub mod diagnostic;
pub mod file_map;
pub mod parser;
pub mod preprocessor;
pub mod symtab;
pub mod typechecker;
pub mod variant;

use crate::{
    ast::TranslationUnit,
    codegen::CodeGen,
    diagnostic::print_diag,
    file_map::{FileMap, source_map},
    preprocessor::{
        include_paths,
        token::{Token, to_string},
    },
};
use ast::printer::Print;
use clap::{Parser, ValueHint};
use codespan_reporting::diagnostic::Diagnostic;
use inkwell::{
    OptimizationLevel,
    context::Context,
    memory_buffer::MemoryBuffer,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};
use lazy_static::lazy_static;
use parser::CParser;
use preprocessor::Preprocessor;
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    sync::Mutex,
};
use symtab::SymbolTable;
use typechecker::TypeChecker;

lazy_static! {
    pub static ref files: Mutex<FileMap> = Mutex::new(FileMap::new());
}

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    pub inputs: Vec<String>,
    #[arg(short, long)]
    pub output: Option<String>,
    #[arg(short = 'I', long = "include-path", value_hint = ValueHint::DirPath)]
    pub include_path: Vec<PathBuf>,
    #[arg(long)]
    pub print_ast: bool,
    #[arg(long)]
    pub print_symtab: bool,
    #[arg(long)]
    pub print_ir: bool,
    /// Preprocess only; do not compile, assemble or link
    #[arg(short = 'E')]
    pub preprocess_only: bool,
    /// Compile only; do not assemble or link
    #[arg(short = 'S')]
    pub compile_only: bool,
    /// Compile and assemble, but do not link
    #[arg(short = 'c')]
    pub compile_and_assemble: bool,
    #[arg(short = 'O')]
    pub optimization_level: Option<u32>,
}

fn get_output(cli: &Cli, input_path: &String, extension: &str) -> PathBuf {
    let mut path = if let Some(t) = &cli.output {
        Path::new(t).to_path_buf()
    } else {
        Path::new(input_path).to_path_buf()
    };
    path.set_extension(extension);
    path
}

fn write(path: &PathBuf, contents: &[u8]) -> Result<(), Diagnostic<usize>> {
    fs::write(&path, contents).map_err(|e| {
        Diagnostic::error().with_message(format!(
            "cannot write '{}': {e}",
            path.to_string_lossy().to_string()
        ))
    })
}

fn preprocess(input_path: &String) -> Result<Vec<Token>, Diagnostic<usize>> {
    let source_id = files.lock().unwrap().add(
        input_path.clone(),
        fs::read_to_string(input_path).map_err(|e| {
            Diagnostic::error().with_message(format!("cannot read '{input_path}': {e}"))
        })?,
    );

    let mut preprocessor = Preprocessor::new(source_id);

    Ok(preprocessor.process()?)
}

fn parse(file_id: usize) -> Result<Rc<RefCell<TranslationUnit>>, Diagnostic<usize>> {
    CParser::new(file_id).parse_to_ast()
}

fn analyze(
    ast: Rc<RefCell<TranslationUnit>>,
) -> Result<Rc<RefCell<SymbolTable>>, Diagnostic<usize>> {
    let symtab = Rc::new(RefCell::new(SymbolTable::new()));

    ast.borrow_mut().symtab = Some(Rc::clone(&symtab));

    let mut type_checker = TypeChecker::new(Rc::clone(&symtab));
    type_checker.check(Rc::clone(&ast))?;

    Ok(symtab)
}

fn gen_code<'ctx>(
    name: &str,
    ast: Rc<RefCell<TranslationUnit>>,
    context: &'ctx Context,
    target_machine: &TargetMachine,
    file_type: FileType,
) -> Result<(CodeGen<'ctx>, MemoryBuffer), Diagnostic<usize>> {
    let module = context.create_module(name);
    module.set_triple(&target_machine.get_triple());

    let mut codegen = CodeGen::new(&context, module);
    codegen.r#gen(Rc::clone(&ast))?;

    let buffer = target_machine
        .write_to_memory_buffer(&codegen.module, file_type)
        .map_err(|e| {
            Diagnostic::error().with_message(format!("cannot generate object file: {e}"))
        })?;

    Ok((codegen, buffer))
}

fn do_frontend(
    cli: &Cli,
    input_path: &String,
    target_machine: &TargetMachine,
) -> Result<Option<PathBuf>, Diagnostic<usize>> {
    let mut output_path = None;
    let mut option_ast = None;
    let mut option_symtab = None;
    let mut option_ir = None;

    match (|| {
        let tokens = preprocess(input_path)?;
        if cli.preprocess_only {
            write(
                &get_output(&cli, input_path, "i"),
                to_string(&tokens).as_bytes(),
            )?;
            return Ok(None);
        }

        let ast = parse(source_map(input_path.clone(), &tokens))?;
        option_ast = Some(Rc::clone(&ast));

        let symtab = analyze(Rc::clone(&ast))?;
        option_symtab = Some(Rc::clone(&symtab));

        let context = Context::create();
        let (codegen, buffer) = gen_code(
            &input_path,
            Rc::clone(&ast),
            &context,
            target_machine,
            if cli.compile_only {
                FileType::Assembly
            } else {
                FileType::Object
            },
        )?;
        option_ir = Some(codegen.module.to_string());

        if cli.compile_only {
            write(&get_output(&cli, &input_path, ".s"), buffer.as_slice())?;
            return Ok(None);
        }

        let output_path = if cli.compile_and_assemble {
            get_output(&cli, &input_path, ".o")
        } else {
            //此时的输出文件是用于链接的
            let mut path = Path::new(input_path).to_path_buf();
            path.set_extension("o");
            path
        };
        write(&output_path, buffer.as_slice())?;

        if cli.compile_and_assemble {
            return Ok(None);
        }

        Ok(Some(output_path))
    })() {
        Ok(t) => output_path = t,
        Err(e) => print_diag(e),
    }

    if cli.print_ast
        && let Some(ast) = option_ast
    {
        ast.print();
    }
    if cli.print_symtab
        && let Some(symtab) = option_symtab
    {
        symtab.borrow().print(0);
    }
    if cli.print_ir
        && let Some(ir) = option_ir
    {
        println!("{ir}");
    }

    Ok(output_path)
}

fn link(_cli: &Cli, mut inputs: Vec<String>) -> Result<(), Diagnostic<usize>> {
    //TODO 链接
    let linker_path = "linker";

    inputs.push("libcmt.lib".to_string());
    inputs.push("oldnames.lib".to_string());

    let args = inputs.clone();

    match Command::new(linker_path).args(&args).spawn() {
        Ok(_) => Ok(()),
        Err(e) => Err(Diagnostic::error()
            .with_message(format!("error occurred when linking: {e}"))
            .with_note(format!("{linker_path} {}", args.join(" ")))),
    }
}

fn main() {
    let cli = Cli::parse();

    include_paths
        .write()
        .unwrap()
        .extend(cli.include_path.clone());

    Target::initialize_all(&InitializationConfig::default());

    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            match cli.optimization_level.unwrap_or(0) {
                1 => OptimizationLevel::Less,
                2 => OptimizationLevel::Default,
                3 | 4 | 5 => OptimizationLevel::Aggressive,
                _ => OptimizationLevel::None,
            },
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    match (|| {
        let mut linker_inputs = vec![];

        for input in &cli.inputs {
            let path = Path::new(input);
            let Some(ext) = path.extension() else {
                continue;
            };
            let ext = ext.to_string_lossy().to_string();
            if ext == "c" {
                if let Some(t) = do_frontend(&cli, input, &target_machine)? {
                    linker_inputs.push(t.to_string_lossy().to_string());
                }
            } else if ext == "o" {
                linker_inputs.push(input.clone());
            }
        }

        if !(cli.preprocess_only || cli.compile_only || cli.compile_and_assemble)
            && linker_inputs.len() > 0
        {
            link(&cli, linker_inputs)?;
        }

        Ok(())
    })() {
        Ok(()) => {}
        Err(e) => print_diag(e),
    }
}
