use std::error::Error;

use quick_xml::events::Event;

use super::{
    hierarchy_tag_store::TagHierarchyStore,
    output_tag_value::OutPutTagValue,
    util::{BytesTag, get_elm_name, get_last_vec_element},
    xml_read_config::{TagValueName, XmlReadConfig},
};

pub fn read_xml(
    reader: &mut quick_xml::Reader<std::io::BufReader<std::fs::File>>,
    read_config: &XmlReadConfig,
) -> Result<Vec<Vec<OutPutTagValue>>, Box<dyn Error>> {
    let read_config: &XmlReadConfig = read_config;

    let mut buf = Vec::new();
    let mut current_tag_hierarchy: Vec<String> = Vec::new();
    let mut tag_store: TagHierarchyStore = TagHierarchyStore::new();

    let mut out_put_value: Vec<OutPutTagValue> = Vec::new();
    let mut out_put_values: Vec<Vec<OutPutTagValue>> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(Box::new(e)),
            Ok(Event::Eof) => break, // ファイルの終端まできたら処理を終了する

            // 開始イベント
            Ok(Event::Start(start)) => {
                let elm_name = get_elm_name(&BytesTag::Start(&start));
                let last_of_current_tag_hierarchy =
                    get_last_vec_element(&current_tag_hierarchy).unwrap_or_default();
                if elm_name != last_of_current_tag_hierarchy {
                    current_tag_hierarchy.push(elm_name.clone());
                }
                if let Some(out_put_tag_value) = tag_store.get_mut(&current_tag_hierarchy) {
                    out_put_tag_value.put_selected_tag_attribute_from_start_tag(
                        &current_tag_hierarchy,
                        &read_config.get_tag_hierarchy_map(),
                        &start,
                    )?;
                }
            }

            // 終了イベント
            Ok(Event::End(end)) => {
                let elm_name = get_elm_name(&BytesTag::End(&end));
                let last_of_current_tag_hierarchy = get_last_vec_element(&current_tag_hierarchy)?;
                if last_of_current_tag_hierarchy == elm_name {
                    if read_config
                        .get_tag_hierarchy_map()
                        .contains_key(&current_tag_hierarchy)
                    {
                        if let Some(out_put_tag_value) = tag_store.get(&current_tag_hierarchy) {
                            out_put_value.push(out_put_tag_value.clone());
                            tag_store.remove(&current_tag_hierarchy);
                        }
                        let tag_group_hierarchy = read_config.get_tag_group_hierarchy();
                        if current_tag_hierarchy == tag_group_hierarchy {
                            out_put_values.push(out_put_value.clone());
                            out_put_value.clear();
                        }
                    }
                    current_tag_hierarchy.pop();
                } else {
                    let s = format!(
                        "tags are mismatched! current_tag_hierarchy : {:?} , elm_name : {:?} !!",
                        current_tag_hierarchy, elm_name
                    );
                    return Err(s.into());
                }
            }

            // テキストイベント
            Ok(Event::Text(e)) => {
                let tag_hierarchy_map = read_config.get_tag_hierarchy_map();
                if let Some(config) = tag_hierarchy_map.get(&current_tag_hierarchy) {
                    let values_list = config.get_target_tag_value_names();
                    let text_value_name = TagValueName::new_tag_text();
                    if values_list.contains(&text_value_name) {
                        let now_text = e.unescape()?.into_owned();
                        if let Some(out_put_tag_value) = tag_store.get_mut(&current_tag_hierarchy) {
                            out_put_tag_value
                                .put_tag_text_value(&current_tag_hierarchy, now_text)?;
                        }
                    }
                }
            }

            // その他のイベントは何もしない
            _ => (),
        }
        buf.clear(); // メモリ節約のためbufをクリアする
    }
    Ok(out_put_values)
}

#[cfg(test)]
mod tests {
    const TV_TAG: &str = "tv";

    const CHANNEL_TAG: &str = "channel";
    const CHANNEL_ID_ATTR: &str = "id";
    const CHANNEL_NUMBER_ATTR: &str = "tp";
    const CHANNEL_SERVICE_ID_ATTR: &str = "service_id";
    const DISPLAY_NAME_TAG: &str = "display-name";

    const PROGRAMME_TAG: &str = "programme";
    const PROGRAMME_START_TIME_ATTR: &str = "start";
    const PROGRAMME_STOP_TIME_ATTR: &str = "stop";
    const PROGRAMME_CHANNEL_ID_ATTR: &str = "channel";
    const PROGRAMME_EVENT_ID_ATTR: &str = "event_id";
    const TITLE_TAG: &str = "title";

    const CATEGORY_TAG: &str = "category";

    const DESCRIPTION_TAG: &str = "desc";

    use std::{fmt::Write, path::PathBuf};

    use log::{error, info};

    use crate::reader::{
        tag_value::TagValue, tag_value_type::TagValueType, util::string_vec,
        xml_read_config::XmlTagReadConfig,
    };

    use super::*;

    const XML_FILE: &str = "test_xml/sample1.xml";

    const DUMP_FLAG: bool = true;

    fn display_output_values(
        out_put_values: &[Vec<OutPutTagValue>],
    ) -> Result<String, Box<dyn Error>> {
        let mut out_put_str = String::new();
        if !DUMP_FLAG {
            return Ok(out_put_str.clone());
        }
        for (i, inner_values) in out_put_values.iter().enumerate() {
            writeln!(&mut out_put_str, "Group {}:", i + 1)?;
            for tag_value in inner_values {
                writeln!(
                    &mut out_put_str,
                    "  Tag Hierarchy: {:?}",
                    tag_value.get_tag_hierarchy()
                )?;
                for value in tag_value.get_tag_values() {
                    writeln!(
                        &mut out_put_str,
                        "    Value Type: {:?}, Name: {}, Value: {}",
                        value.get_value_type(),
                        value.get_tag_name(),
                        value.get_value()
                    )?;
                }
            }
        }
        Ok(out_put_str.clone())
    }

    #[test_log::test]
    fn test_read_xml_channel() {
        let check_tag_hierarchy_1: Vec<String> = string_vec(vec![TV_TAG, CHANNEL_TAG]);
        let mut x: Vec<String> = check_tag_hierarchy_1.clone();
        x.push(DISPLAY_NAME_TAG.to_string());
        let check_tag_hierarchy_2: Vec<String> = x;

        let check_tag_value_display_name_1_1: TagValue = TagValue::new(
            TagValueType::TagText,
            DISPLAY_NAME_TAG.to_string(),
            "CHANNEL_NAME_1_1".to_string(),
        );
        let check_tag_value_channel_1_1_1: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            CHANNEL_ID_ATTR.to_string(),
            "GR2_1032".to_string(),
        );
        let check_tag_value_channel_1_1_2: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            CHANNEL_SERVICE_ID_ATTR.to_string(),
            "1032".to_string(),
        );
        let check_tag_value_channel_1_1_3: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            CHANNEL_NUMBER_ATTR.to_string(),
            "26".to_string(),
        );
        let check_tag_value_display_name_1_2: TagValue = TagValue::new(
            TagValueType::TagText,
            DISPLAY_NAME_TAG.to_string(),
            "CHANNEL_NAME_1_2".to_string(),
        );
        let check_tag_value_channel_1_2_1: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            CHANNEL_ID_ATTR.to_string(),
            "GR2_1034".to_string(),
        );
        let check_tag_value_channel_1_2_2: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            CHANNEL_SERVICE_ID_ATTR.to_string(),
            "1034".to_string(),
        );
        let check_tag_value_channel_1_2_3: TagValue = check_tag_value_channel_1_1_3.clone();

        let xml_file = PathBuf::from(XML_FILE);
        let mut reader: quick_xml::Reader<std::io::BufReader<std::fs::File>> =
            quick_xml::Reader::from_reader(std::io::BufReader::new(
                std::fs::File::open(xml_file).unwrap(),
            ));

        let mut config = XmlReadConfig::new(check_tag_hierarchy_1.clone());

        let tag_value_list_channel = XmlTagReadConfig::new(
            check_tag_hierarchy_1.clone(),
            std::collections::HashSet::from_iter(vec![
                TagValueName::new_attribute(CHANNEL_ID_ATTR.to_string()),
                TagValueName::new_attribute(CHANNEL_SERVICE_ID_ATTR.to_string()),
                TagValueName::new_attribute(CHANNEL_NUMBER_ATTR.to_string()),
            ]),
        );
        if let Err(e) = config.insert_xml_tag_read_config_to_hash_map(&tag_value_list_channel) {
            error!("Error: {}", e);
        }

        let tag_value_list_channel_display_name = XmlTagReadConfig::new(
            check_tag_hierarchy_2.clone(),
            std::collections::HashSet::from_iter(vec![TagValueName::new_tag_text()]),
        );
        if let Err(e) =
            config.insert_xml_tag_read_config_to_hash_map(&tag_value_list_channel_display_name)
        {
            error!("Error: {}", e);
        }

        let res: Result<Vec<Vec<OutPutTagValue>>, Box<dyn Error>> = read_xml(&mut reader, &config);
        if let Err(e) = res {
            error!("Error: {}", e);
        } else {
            let out_put_values: Vec<Vec<OutPutTagValue>> = res.unwrap();

            info!("{}", display_output_values(&out_put_values).unwrap());

            //チャンネルが2件取得できたことを確認する。
            assert_eq!(out_put_values.len(), 2);

            //channelタグの情報とchannelタグの子タグ(display-name)で2件が取得できていることを確認する。
            for inner_values in out_put_values.iter() {
                assert_eq!(inner_values.len(), 2);
            }
            for inner_values in out_put_values.iter() {
                for tag_value in inner_values.iter() {
                    let tag_hierarchy = tag_value.get_tag_hierarchy();
                    if tag_hierarchy == check_tag_hierarchy_1 {
                        let tag_values = tag_value.get_tag_values();
                        assert_eq!(tag_values.len(), 3);
                        let id_assert_1_1: bool =
                            tag_values.contains(&check_tag_value_channel_1_1_1);
                        info!("id_assert_1_1 {:?}", id_assert_1_1);
                        let service_id_assert_1_1: bool =
                            tag_values.contains(&check_tag_value_channel_1_1_2);
                        info!("service_id_assert_1_1 {:?}", service_id_assert_1_1);
                        let tp_assert_1_1: bool =
                            tag_values.contains(&check_tag_value_channel_1_1_3);
                        info!("tp_assert_1_1 {:?}", tp_assert_1_1);
                        let ch_1_1 = id_assert_1_1 && service_id_assert_1_1 && tp_assert_1_1;
                        info!("ch_1_1 {:?}", ch_1_1);
                        let id_assert_1_2: bool =
                            tag_values.contains(&check_tag_value_channel_1_2_1);
                        info!("id_assert_1_2 {:?}", id_assert_1_2);
                        let service_id_assert_1_2: bool =
                            tag_values.contains(&check_tag_value_channel_1_2_2);
                        info!("service_id_assert_1_2 {:?}", service_id_assert_1_2);
                        let tp_assert_1_2: bool =
                            tag_values.contains(&check_tag_value_channel_1_2_3);
                        info!("tp_assert_1_2 {:?}", tp_assert_1_2);
                        let ch_1_2 = id_assert_1_2 && service_id_assert_1_2 && tp_assert_1_2;
                        info!("ch_1_2 {:?}", ch_1_2);
                        let ch = ch_1_1 ^ ch_1_2;
                        info!("ch {:?}", ch);
                        assert!(ch);
                    } else if tag_hierarchy == check_tag_hierarchy_2 {
                        let tag_values = tag_value.get_tag_values();
                        assert_eq!(tag_values.len(), 1);
                        let display_name_assert_1_1: bool =
                            tag_values.contains(&check_tag_value_display_name_1_1);
                        info!("display_name_assert_1_1 {:?}", display_name_assert_1_1);
                        let display_name_assert_1_2: bool =
                            tag_values.contains(&check_tag_value_display_name_1_2);
                        info!("display_name_assert_1_2 {:?}", display_name_assert_1_2);
                        let display_name = display_name_assert_1_1 ^ display_name_assert_1_2;
                        info!("display_name {:?}", display_name);
                        assert!(display_name);
                    } else {
                        panic!("Unknown tag hierarchy: {:?}", tag_hierarchy);
                    }
                }
            }
        }
    }

    #[test_log::test]
    fn test_read_xml_programme() {
        let xml_file = PathBuf::from(XML_FILE);
        let mut reader: quick_xml::Reader<std::io::BufReader<std::fs::File>> =
            quick_xml::Reader::from_reader(std::io::BufReader::new(
                std::fs::File::open(xml_file).unwrap(),
            ));

        let check_tag_hierarchy_1 = string_vec(vec![TV_TAG, PROGRAMME_TAG]);
        let mut x = check_tag_hierarchy_1.clone();
        x.push(TITLE_TAG.to_string());
        let check_tag_hierarchy_2 = x;
        let mut y = check_tag_hierarchy_1.clone();
        y.push(DESCRIPTION_TAG.to_string());
        let check_tag_hierarchy_3 = y;
        let mut z = check_tag_hierarchy_1.clone();
        z.push(CATEGORY_TAG.to_string());
        let check_tag_hierarchy_4 = z;

        let check_tag_value_title_1: TagValue = TagValue::new(
            TagValueType::TagText,
            TITLE_TAG.to_string(),
            "TITLE_1_1".to_string(),
        );
        let check_tag_value_desc_1: TagValue = TagValue::new(
            TagValueType::TagText,
            DESCRIPTION_TAG.to_string(),
            "DESC_1".to_string(),
        );
        let check_tag_value_category_1_1: TagValue = TagValue::new(
            TagValueType::TagText,
            CATEGORY_TAG.to_string(),
            "CAT_1".to_string(),
        );
        let check_tag_value_category_1_2: TagValue = TagValue::new(
            TagValueType::TagText,
            CATEGORY_TAG.to_string(),
            "CAT_2".to_string(),
        );
        let check_tag_value_start_1: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            PROGRAMME_START_TIME_ATTR.to_string(),
            "20241123090000 +0900".to_string(),
        );
        let check_tag_value_stop_1: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            PROGRAMME_STOP_TIME_ATTR.to_string(),
            "20241123092000 +0900".to_string(),
        );
        let check_tag_value_channel_1: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            PROGRAMME_CHANNEL_ID_ATTR.to_string(),
            "GR2_1032".to_string(),
        );
        let check_tag_value_event_id_1: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            PROGRAMME_EVENT_ID_ATTR.to_string(),
            "32665".to_string(),
        );
        let check_tag_value_title_2: TagValue = TagValue::new(
            TagValueType::TagText,
            TITLE_TAG.to_string(),
            "TITLE_1_2".to_string(),
        );
        let check_tag_value_desc_2: TagValue = check_tag_value_desc_1.clone();
        let check_tag_value_category_2_1: TagValue = check_tag_value_category_1_1.clone();
        let check_tag_value_category_2_2: TagValue = check_tag_value_category_1_2.clone();
        let check_tag_value_start_2: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            PROGRAMME_START_TIME_ATTR.to_string(),
            "20241123090000 +0900".to_string(),
        );
        let check_tag_value_stop_2: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            PROGRAMME_STOP_TIME_ATTR.to_string(),
            "20241123092000 +0900".to_string(),
        );
        let check_tag_value_channel_2: TagValue = check_tag_value_channel_1.clone();
        let check_tag_value_event_id_2: TagValue = TagValue::new(
            TagValueType::AttributeValue,
            PROGRAMME_EVENT_ID_ATTR.to_string(),
            "32666".to_string(),
        );

        let mut config = XmlReadConfig::new(check_tag_hierarchy_1.clone());

        let tag_value_list_programme_master = XmlTagReadConfig::new(
            check_tag_hierarchy_1.clone(),
            std::collections::HashSet::from_iter(vec![
                TagValueName::new_attribute(PROGRAMME_START_TIME_ATTR.to_string()),
                TagValueName::new_attribute(PROGRAMME_STOP_TIME_ATTR.to_string()),
                TagValueName::new_attribute(PROGRAMME_CHANNEL_ID_ATTR.to_string()),
                TagValueName::new_attribute(PROGRAMME_EVENT_ID_ATTR.to_string()),
            ]),
        );
        if let Err(e) =
            config.insert_xml_tag_read_config_to_hash_map(&tag_value_list_programme_master)
        {
            error!("Error: {}", e);
        }

        let tag_value_list_programme_title = XmlTagReadConfig::new(
            check_tag_hierarchy_2.clone(),
            std::collections::HashSet::from_iter(vec![TagValueName::new_tag_text()]),
        );
        if let Err(e) =
            config.insert_xml_tag_read_config_to_hash_map(&tag_value_list_programme_title)
        {
            error!("Error: {}", e);
        }

        let tag_value_list_programme_desc = XmlTagReadConfig::new(
            check_tag_hierarchy_3.clone(),
            std::collections::HashSet::from_iter(vec![TagValueName::new_tag_text()]),
        );
        if let Err(e) =
            config.insert_xml_tag_read_config_to_hash_map(&tag_value_list_programme_desc)
        {
            error!("Error: {}", e);
        }

        let tag_value_list_programme_category = XmlTagReadConfig::new(
            check_tag_hierarchy_4.clone(),
            std::collections::HashSet::from_iter(vec![TagValueName::new_tag_text()]),
        );
        if let Err(e) =
            config.insert_xml_tag_read_config_to_hash_map(&tag_value_list_programme_category)
        {
            error!("Error: {}", e);
        }

        let res: Result<Vec<Vec<OutPutTagValue>>, Box<dyn Error>> = read_xml(&mut reader, &config);
        if let Err(e) = res {
            error!("Error: {}", e);
        } else {
            let out_put_values: Vec<Vec<OutPutTagValue>> = res.unwrap();
            info!("{}", display_output_values(&out_put_values).unwrap());

            //番組が2件取得できたことを確認する。
            assert_eq!(out_put_values.len(), 2);

            //programmeタグの情報とprogrammeタグの子タグで5件が取得できていることを確認する。
            for inner_values in out_put_values.iter() {
                assert_eq!(inner_values.len(), 5);
            }

            for inner_values in out_put_values.iter() {
                for tag_value in inner_values.iter() {
                    let tag_hierarchy = tag_value.get_tag_hierarchy();
                    if tag_hierarchy == check_tag_hierarchy_1 {
                        let tag_values = tag_value.get_tag_values();
                        assert_eq!(tag_values.len(), 4);
                        let start_assert_1: bool = tag_values.contains(&check_tag_value_start_1);
                        let start_assert_2: bool = tag_values.contains(&check_tag_value_start_2);
                        let stop_assert_1: bool = tag_values.contains(&check_tag_value_stop_1);
                        let stop_assert_2: bool = tag_values.contains(&check_tag_value_stop_2);
                        let channel_assert_1: bool =
                            tag_values.contains(&check_tag_value_channel_1);
                        let channel_assert_2: bool =
                            tag_values.contains(&check_tag_value_channel_2);
                        let event_id_assert_1: bool =
                            tag_values.contains(&check_tag_value_event_id_1);
                        let event_id_assert_2: bool =
                            tag_values.contains(&check_tag_value_event_id_2);
                        let programme_1 = start_assert_1
                            && stop_assert_1
                            && channel_assert_1
                            && event_id_assert_1;
                        info!("programme {:?}", programme_1);
                        let programme_2 = start_assert_2
                            && stop_assert_2
                            && channel_assert_2
                            && event_id_assert_2;
                        info!("programme {:?}", programme_2);
                        let programme = programme_1 ^ programme_2;
                        info!("programme {:?}", programme);
                        assert!(programme);
                    } else if tag_hierarchy == check_tag_hierarchy_2 {
                        let tag_values = tag_value.get_tag_values();
                        assert_eq!(tag_values.len(), 1);
                        let title_assert_1: bool = tag_values.contains(&check_tag_value_title_1);
                        info!("title_assert_1 {:?}", title_assert_1);
                        let title_assert_2: bool = tag_values.contains(&check_tag_value_title_2);
                        info!("title_assert_2 {:?}", title_assert_2);
                        let title = title_assert_1 ^ title_assert_2;
                        info!("title {:?}", title);
                        assert!(title);
                    } else if tag_hierarchy == check_tag_hierarchy_3 {
                        let tag_values = tag_value.get_tag_values();
                        assert_eq!(tag_values.len(), 1);
                        let desc_assert: bool = tag_values.contains(&check_tag_value_desc_1)
                            || tag_values.contains(&check_tag_value_desc_2);
                        info!("desc_assert {:?}", desc_assert);
                        assert!(desc_assert);
                    } else if tag_hierarchy == check_tag_hierarchy_4 {
                        let tag_values = tag_value.get_tag_values();
                        assert_eq!(tag_values.len(), 1);
                        let category_assert_1: bool = tag_values
                            .contains(&check_tag_value_category_1_1)
                            ^ tag_values.contains(&check_tag_value_category_1_2);
                        info!("category_assert_1 {:?}", category_assert_1);
                        let category_assert_2: bool = tag_values
                            .contains(&check_tag_value_category_2_1)
                            ^ tag_values.contains(&check_tag_value_category_2_2);
                        info!("category_assert_2 {:?}", category_assert_2);
                        let category = category_assert_1 && category_assert_2;
                        info!("category {:?}", category);
                        assert!(category);
                    } else {
                        panic!("Unknown tag hierarchy: {:?}", tag_hierarchy);
                    }
                }
            }
        }
    }
}
