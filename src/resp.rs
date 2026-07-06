use crate::resp_result::RESPResult;
use crate::resp_result::RESPError;


#[cfg(test)]
mod tests{

use std::process::Output;

use crate::resp_result::{RESPError,RESPResult};

use super::*;

    #[test]
    fn test_binary_extract_line(){
        let buffer = "OK\r\n".as_bytes();
        let mut index:usize = 0;
        let output = binary_extract_line(buffer, &mut index).unwrap();
        assert_eq!(output, "OK".as_bytes());
        assert_eq!(index,4);



    }
    #[test]
    fn test_binary_extract_line_loger_string(){
        let buffer = "ECHO\r\n".as_bytes();
        let mut index:usize= 0;
        let Output = binary_extract_line(buffer, &mut index).unwrap();
        assert_eq!(Output,"ECHO".as_bytes());
        assert!(index==6);


    }
    #[test]
    fn test_binary_extract_line_empty_buffer(){
        let buffer = "".as_bytes();
        let mut index:usize = 0;
        match binary_extract_line(buffer, &mut index){
            Err(RESPError::OutOfBounds(index))=>{
                assert_eq!(index,0);
            }
            _=>panic!("unexpected error"),

        }

    }
    #[test]
    fn test_binary_extract_line_no_separator(){
        let buffer = "OK".as_bytes();
        let mut index:usize = 0;
        match binary_extract_line(buffer, &mut index){
            Err(RESPError::OutOfBounds(index))=>{
                assert_eq!(index,2)
            }
            _=>panic!(),
        }
    }
    #[test]
    fn test_binary_extract_line_index_too_advanced(){
        let buffer = "OK".as_bytes();
        let mut index:usize =1;
        match  binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index))=>{
                assert_eq!(index,2);
            }
            _=>panic!(),
        }
    }
    #[test]
   fn test_binary_extract_line_as_string(){
        let buffer = "OK\r\n".as_bytes();
        let mut index:usize = 0;
        let output = binary_extract_line_as_string(buffer, &mut index).unwrap();
        assert_eq!(output,String::from("OK"));
        assert_eq!(index,4);
        
    }
}

fn binary_extract_line(buffer:&[u8],index :&mut usize)->RESPResult<Vec<u8>>{


let mut output = Vec::new();
if *index>=buffer.len(){
    return Err(RESPError::OutOfBounds(*index));
}
let mut previous_elem:u8 = buffer[*index].clone();
let mut final_index:usize = *index;
let mut separator_found:bool = false;
for &elem in buffer[*index..].iter(){
     final_index += 1;
    if elem == b'\n' && previous_elem == b'\r'{
        separator_found =true;
        break;
    }
    previous_elem = elem;
   
}
if !separator_found{
    *index = final_index;
    return Err(RESPError::OutOfBounds(*index))
}
output.extend_from_slice(&buffer[*index..final_index-2]);
*index = final_index;
Ok(output)


}
fn binary_extract_line_as_string(buffer:&[u8],index :&mut usize)->RESPResult<String>{
    let line = binary_extract_line(buffer, index)?;
    Ok(String::from_utf8(line)?)

}
