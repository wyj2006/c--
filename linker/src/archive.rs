use anyhow::Result;

pub static ARCHIVE_MAGIC: &[u8; 8] = b"!<arch>\n";

#[derive(Debug)]
pub struct Archive {
    pub members: Vec<Member>,
    pub string_table: Vec<u8>,
}

#[derive(Debug)]
pub struct Member {
    pub name: String,
    pub date: String,
    pub user_id: String,
    pub group_id: String,
    pub mode: String,
    pub data: Vec<u8>,
}

impl Archive {
    pub fn parse(data: &Vec<u8>) -> Result<Archive> {
        let mut string_table = vec![];
        let mut members = vec![];

        let mut i = 8;

        while i < data.len() {
            let header = &data[i..i + 60];
            let mut name = String::from_utf8_lossy(&header[..16]).to_string();
            let date = String::from_utf8_lossy(&header[16..16 + 12]).to_string();
            let user_id = String::from_utf8_lossy(&header[28..28 + 6]).to_string();
            let group_id = String::from_utf8_lossy(&header[34..34 + 6]).to_string();
            let mode = String::from_utf8_lossy(&header[40..40 + 8]).to_string();
            let size = String::from_utf8_lossy(&header[48..48 + 10])
                .to_string()
                .replace(" ", "")
                .parse::<usize>()?;

            i += 60;
            let data = data[i..i + size].to_vec();
            i += size;
            if size % 2 == 1 {
                i += 1;
            }

            if name.replace(" ", "") == "//" {
                string_table.extend(data);
                continue;
            }
            if name.replace(" ", "") == "/" {
                //符号表
                continue;
            }
            if name.starts_with("/") {
                let start = name[1..].replace(" ", "").parse::<usize>()?;
                let mut end = start + 1;
                while end < string_table.len() && string_table[end] != b'\0' {
                    end += 1;
                }
                name = String::from_utf8_lossy(&string_table[start..end]).to_string();
            }
            members.push(Member {
                name,
                date,
                user_id,
                group_id,
                mode,
                data,
            });
        }
        Ok(Archive {
            members,
            string_table,
        })
    }
}
