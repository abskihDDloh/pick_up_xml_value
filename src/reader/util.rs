use std::error::Error;

use quick_xml::events::{BytesEnd, BytesStart};

pub(in crate::reader) enum BytesTag<'a> {
    Start(&'a BytesStart<'a>),
    End(&'a BytesEnd<'a>),
}

pub(in crate::reader) fn get_elm_name(tag: &BytesTag) -> String {
    match tag {
        BytesTag::Start(tag) => String::from_utf8(tag.name().as_ref().to_vec()).unwrap(),
        BytesTag::End(tag) => String::from_utf8(tag.name().as_ref().to_vec()).unwrap(),
    }
}

/// Vec<String>の最後の要素を取得します。
/// 取得できない場合はエラーを返します。
pub fn get_last_vec_element(src: &Vec<String>) -> Result<String, Box<dyn Error>> {
    let s = src.last();
    match s {
        Some(tag_name) => Ok(tag_name.clone()),
        None => {
            let s_err = format!(
                "
                SRC VEC IS EMPTY. src {:?}",
                src
            );
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                s_err,
            )))
        }
    }
}
