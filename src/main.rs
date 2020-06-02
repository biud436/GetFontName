use std::env;
use std::io::prelude::*;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::SeekFrom;
use std::str;
use encoding_rs::*;

struct OffsetTable {
    major_version:u16,
    minor_version:u16,
    num_of_tables:u16,
    padding:u16
}

struct NameHeader {
    format_selector:u16,
    name_record_count:u16,
    storage_offset:u16
}

#[derive(Debug)]
struct NameRecord {
    platform_id:u16,
    encoding_id:u16,
    language_id:u16,
    name_id:u16,
    string_length:u16,
    string_offset:u16,
    name:String,
}

fn main() {
    println!("Hello, world!");

    // 러스트에는 값(리터럴) 타입과 객체 타입이 있다.
    // 객체 타입은 힙에 할당된다.
    let mut font_name:String = String::from("");

    for argument in env::args() {
        if argument.contains("--font=") {

            // 러스트는 소유권 개념이 엄격하다.
            // 아래 변수는 지역적이기 때문에 밖으로 내보낼 수 없다.
            let v: Vec<&str> = argument.split("=").collect();
            
            // 지역 변수이기 때문에 for 문이 끝나면 제거된다.
            // 따라서 힙에 할당되어있는 문자열 객체로 선언해야 한다.
            // font_name = v[1]과 같은 식으로는 동작하지 않는다.
            font_name = String::from(v[1]);
        }
    }

    println!("{}", font_name);
    let path = Path::new(&font_name);
    let display = path.display();

    // 파일을 생성한다.
    let mut f = match File::open(&path) {
        Err(why) => panic!("error {} {}", display, why.to_string()),
        Ok(file) => file,
    };

    let metadata = std::fs::metadata(&font_name).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    // 버퍼를 읽고 struct를 채운다.
    let mut rdr = Cursor::new(buffer);
    let offset_table = OffsetTable {
        major_version: rdr.read_u16::<BigEndian>().unwrap(),
        minor_version: rdr.read_u16::<BigEndian>().unwrap(),
        num_of_tables: rdr.read_u16::<BigEndian>().unwrap(),
        padding: rdr.read_u16::<BigEndian>().unwrap()
    };

    // 버전을 확인한다.
    println!("offset_table.major_version : {}", offset_table.major_version);
    println!("offset_table.major_version : {}", offset_table.minor_version);
    println!("offset_table.major_version : {}", offset_table.num_of_tables);
    println!("offset_table.major_version : {}", offset_table.padding);

    // 4바이트 스킵
    rdr.seek(SeekFrom::Current(4)).unwrap();

    // 현재 위치가 12이면 제대로된 위치이다.
    println!("Position : {}", rdr.position());

    if offset_table.major_version != 1 || offset_table.minor_version != 0 {
        panic!("This font is not True Type Font");
    }

    let mut name_table_offset = 0x00;
    // let mut name_table_length = 0x00;
    let mut is_found_name_table = false;

    for _i in 0..offset_table.num_of_tables {
        let mut tag_buffer = vec![0; 4 as usize];
        rdr.read(&mut tag_buffer).unwrap();
        let tag_name = str::from_utf8(&tag_buffer).expect("Found invalid UTF-8");
        
        let _check_sum = rdr.read_u32::<BigEndian>().unwrap();
        let offset = rdr.read_u32::<BigEndian>().unwrap();
        let _length = rdr.read_u32::<BigEndian>().unwrap();

        if "name" == tag_name {
            println!("found {}", tag_name);
            name_table_offset = offset;
            // name_table_length = length;
            is_found_name_table = true;
            break;
        }
    }

    if !is_found_name_table {
        panic!("Cannot find the name table.");
    }

    rdr.set_position(name_table_offset as u64);

    println!("current position : {}", rdr.position());

    // NameHeader 생성
    let name_header = NameHeader {
        format_selector:rdr.read_u16::<BigEndian>().unwrap(),
        name_record_count :rdr.read_u16::<BigEndian>().unwrap(),
        storage_offset :rdr.read_u16::<BigEndian>().unwrap(),
    };

    if name_header.format_selector == 1 {
        println!("langTagCount detect");        
        println!("langTagRecord[langTagCount] detect");        
    }

    let mut name_record_table:Vec<NameRecord> = Vec::new();

    for _i in 0..name_header.name_record_count {
        let mut name_record = NameRecord {
            platform_id: rdr.read_u16::<BigEndian>().unwrap(),
            encoding_id: rdr.read_u16::<BigEndian>().unwrap(),
            language_id: rdr.read_u16::<BigEndian>().unwrap(),
            name_id: rdr.read_u16::<BigEndian>().unwrap(),
            string_length: rdr.read_u16::<BigEndian>().unwrap(),
            string_offset: rdr.read_u16::<BigEndian>().unwrap(),
            name: String::default(),
        };

        // 폰트 패밀리 취득
        if name_record.name_id == 4 {
            let temp_file_pos:u64 = rdr.position();
            rdr.set_position(name_table_offset as u64 + name_record.string_offset as u64 + name_header.storage_offset as u64);

            let mut name_buffer = vec![0; name_record.string_length as usize];
            rdr.read(&mut name_buffer).unwrap();
            
            let (res, _enc, _errors) = EUC_KR.decode(&name_buffer);
            name_record.name = String::from(res);
            name_record_table.push(name_record);
            rdr.set_position(temp_file_pos);
        }
    }

    for record in name_record_table {
        println!("Font Name : {}", record.name);
    }

}