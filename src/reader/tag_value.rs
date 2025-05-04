use super::tag_value_type::TagValueType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagValue {
    /// 種類
    value_type: TagValueType,
    /// 名前(タグのテキストであればタグ名が入る。属性の場合は属性名が入る。)
    name: String,
    /// 値
    value: String,
}
impl TagValue {
    pub fn new(value_type: TagValueType, name: String, value: String) -> Self {
        TagValue {
            value_type,
            name,
            value,
        }
    }

    /// タグ名を取得します。
    pub fn get_tag_name(&self) -> &str {
        &self.name
    }

    /// 値を取得します。
    pub fn get_value(&self) -> &str {
        &self.value
    }

    /// タグの種類を取得します。
    pub fn get_value_type(&self) -> &TagValueType {
        &self.value_type
    }
}
