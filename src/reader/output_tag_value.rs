use std::collections::HashMap;
use std::{collections::HashSet, error::Error};

use log::{debug, warn};
use quick_xml::events::BytesStart;

use super::tag_value::TagValue;
use super::tag_value_type::TagValueType::{AttributeValue, TagText};
use super::util::get_last_vec_element;
use super::xml_read_config::XmlTagReadConfig;

/// `OutPutValue` は、XML のタグや属性の情報を格納する構造体です。
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct OutPutTagValue {
    /// タグ階層
    tag_hierarchy: Vec<String>,
    /// 値のリスト
    tag_values: Vec<TagValue>,
}
impl OutPutTagValue {
    /// タグ名(タグ階層の最後のタグ名)を取得します。
    pub fn get_tag_name(&self) -> Result<String, Box<dyn Error>> {
        get_last_vec_element(&self.tag_hierarchy)
    }

    /// タグ階層のコピーを取得します。
    pub fn get_tag_hierarchy(&self) -> Vec<String> {
        self.tag_hierarchy.clone()
    }

    /// タグの値のリストが空かどうかを確認します。
    /// 空の場合は true を返します。
    /// 空でない場合は false を返します。
    pub fn is_tag_values_empty(&self) -> bool {
        self.tag_values.is_empty()
    }

    /// タグの値のリストを取得します。
    pub fn get_tag_values(&self) -> Vec<TagValue> {
        self.tag_values.clone()
    }

    /// タグ階層を設定します。
    /// タグ階層が空の場合のみ設定します。
    /// タグ階層が空でない場合は、現在のタグ階層と新しいタグ階層を比較します。
    /// タグ階層が異なる場合は、エラーを返します。
    /// タグ階層が同じ場合は、何もしません。
    pub(in crate::reader) fn set_tag_hierarchy(
        &mut self,
        tag_hitrarchy: &Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        let tag_hitrarchy_clone = tag_hitrarchy.clone();
        if self.tag_hierarchy.is_empty() {
            self.tag_hierarchy = tag_hitrarchy_clone;
            Ok(())
        } else if self.tag_hierarchy != tag_hitrarchy_clone {
            let s_err = format!(
                "
                TAG HIERARCHY IS DIFFERENT. tag_hitrarchy {:?} != self.tag_hitrarchy {:?}",
                tag_hitrarchy, self.tag_hierarchy
            );
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                s_err,
            )))
        } else {
            Ok(())
        }
    }

    /// タグの値を追加します。
    /// タグ階層が空の場合は、タグ階層を設定します。
    /// タグ階層が空でない場合は、現在のタグ階層と新しいタグ階層を比較します。
    /// タグ階層が異なる場合は、エラーを返します。
    /// タグ階層が同じ場合は、値を追加します。(名前と値が同じ場合は追加しない)
    fn put_tag_value(
        &mut self,
        tag_hierarchy: &Vec<String>,
        value: TagValue,
    ) -> Result<(), Box<dyn Error>> {
        {
            self.set_tag_hierarchy(tag_hierarchy)?;
            // 重複を避けるために HashSet を使用
            let mut unique_values: HashSet<TagValue> = HashSet::new();
            for val in &self.tag_values {
                unique_values.insert(val.clone());
            }
            if !unique_values.contains(&value) {
                self.tag_values.push(value);
            }
            Ok(())
        }
    }

    pub(in crate::reader) fn put_tag_text_value(
        &mut self,
        tag_hierarchy: &Vec<String>,
        text_value: String,
    ) -> Result<(), Box<dyn Error>> {
        let tag_name_from_tag_hierarchy = self.get_tag_name()?;
        let out_put_text_event_value =
            TagValue::new(TagText, tag_name_from_tag_hierarchy, text_value);
        self.put_tag_value(tag_hierarchy, out_put_text_event_value)
    }
    /*
              pub(in crate::pick_up_xml_value
    ::reader::common)  fn put_all_tag_attribute_from_start_tag(
                &mut self,
                tag_hierarchy: &Vec<String>,
                start: &BytesStart,
            ) -> Result<(), Box<dyn Error>> {
                let tag_name_from_start_tag = String::from_utf8(start.name().as_ref().to_vec())?;
                let tag_name_from_tag_hierarchy = self.get_tag_name()?;
                if tag_name_from_tag_hierarchy != tag_name_from_start_tag {
                    let s_err = format!(
                        "TAG NAME IS DIFFERENT. tag_hierarchy {:?} , tag_name_from_start_tag {:?} != tag_name_from_tag_hierarchy {:?}",
                        self.tag_hierarchy, tag_name_from_start_tag, tag_name_from_tag_hierarchy
                    );
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        s_err,
                    )));
                }
                for attr in start.attributes().flatten() {
                    let attr_name = match String::from_utf8(attr.key.as_ref().to_vec()) {
                        Ok(name) => name,
                        Err(e) => {
                            warn!("FAILED TO CONVERT ATTRIBUTE NAME TO STRING: {}", e);
                            continue;
                        }
                    };
                    let attr_value = match String::from_utf8(attr.value.to_vec()) {
                        Ok(value) => value,
                        Err(e) => {
                            warn!("FAILED TO CONVERT ATTRIBUTE VALUE TO STRING: {}", e);
                            continue;
                        }
                    };
                    let out_put_attrubute_value = TagValue::new(AttributeValue, attr_name, attr_value);
                    self.put_tag_value(tag_hierarchy, out_put_attrubute_value)?;
                }
                Ok(())
            }
        */
    pub(in crate::reader) fn put_selected_tag_attribute_from_start_tag(
        &mut self,
        tag_hierarchy: &Vec<String>,
        read_config: &HashMap<Vec<String>, XmlTagReadConfig>,
        start: &BytesStart,
    ) -> Result<(), Box<dyn Error>> {
        let tag_name_from_start_tag = String::from_utf8(start.name().as_ref().to_vec())?;
        let tag_name_from_tag_hierarchy = self.get_tag_name()?;
        if tag_name_from_tag_hierarchy != tag_name_from_start_tag {
            let s_err = format!(
                "TAG NAME IS DIFFERENT. tag_hierarchy {:?} , tag_name_from_start_tag {:?} != tag_name_from_tag_hierarchy {:?}",
                self.tag_hierarchy, tag_name_from_start_tag, tag_name_from_tag_hierarchy
            );
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                s_err,
            )));
        }
        for attr in start.attributes().flatten() {
            #[allow(unused_assignments)]
            let mut attr_name = String::new();
            attr_name = match String::from_utf8(attr.key.as_ref().to_vec()) {
                Ok(name) => name,
                Err(e) => {
                    warn!("FAILED TO CONVERT ATTRIBUTE NAME TO STRING: {}", e);
                    continue;
                }
            };
            let config_o: Option<&XmlTagReadConfig> = read_config.get(tag_hierarchy);
            let config = match config_o {
                Some(v) => v,
                None => {
                    debug!("ATTRIBUTE LIST NOT FOUND.");
                    continue;
                }
            };
            let attribute_list = config.get_target_tag_value_names();
            for attr_value_name in attribute_list {
                if (attr_value_name.get_value_type() == AttributeValue)
                    && (attr_value_name.get_name() == attr_name)
                {
                    let attr_value = match String::from_utf8(attr.value.to_vec()) {
                        Ok(value) => value,
                        Err(e) => {
                            warn!("FAILED TO CONVERT ATTRIBUTE VALUE TO STRING: {}", e);
                            continue;
                        }
                    };
                    let out_put_attrubute_value =
                        TagValue::new(AttributeValue, attr_name.clone(), attr_value);
                    self.put_tag_value(tag_hierarchy, out_put_attrubute_value)?;
                }
            }
        }
        Ok(())
    }
}
