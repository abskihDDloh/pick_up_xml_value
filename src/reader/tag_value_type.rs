#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TagValueType {
    /// タグのテキスト
    TagText,
    /// タグの属性
    AttributeValue,
}
