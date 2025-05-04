use std::collections::{HashMap, HashSet};

use super::tag_value_type::TagValueType;

/// `TagValueName`は、XMLタグの値（属性またはテキスト）を表します。
///
/// この構造体は、XMLタグの属性値またはテキスト値を表現するために使用されます。
/// 属性の場合は名前を持ち、テキスト値の場合は名前は空文字列になります。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TagValueName {
    /// 種類（属性値またはタグテキスト）
    value_type: TagValueType,
    /// 名前（タグのテキストの場合は空文字列、属性の場合は属性名）
    name: String,
}

impl TagValueName {
    /// タグのテキスト値を表す新しい`TagValueName`を作成します。
    ///
    /// # 戻り値
    /// - タグテキストを表す`TagValueName`インスタンス
    ///
    pub fn new_tag_text() -> Self {
        TagValueName {
            value_type: TagValueType::TagText,
            name: String::new(),
        }
    }

    /// 属性値を表す新しい`TagValueName`を作成します。
    ///
    /// # 引数
    /// - `name`: 属性名
    ///
    /// # 戻り値
    /// - 属性値を表す`TagValueName`インスタンス
    ///
    pub fn new_attribute(name: String) -> Self {
        TagValueName {
            value_type: TagValueType::AttributeValue,
            name,
        }
    }

    /// 名前を取得します。
    ///
    /// # 戻り値
    /// - 属性名または空文字列（タグテキストの場合）
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// 種類を取得します。
    ///
    /// # 戻り値
    /// - `TagValueType`（属性値またはタグテキスト）
    pub fn get_value_type(&self) -> TagValueType {
        self.value_type.clone()
    }
}

/// `XmlTagReadConfig`は、XMLタグの読み取り設定を表します。
///
/// この構造体は、特定のXMLタグ階層に関連付けられた読み取り設定を保持します。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XmlTagReadConfig {
    /// タグ階層（例: `["root", "child", "subchild"]`）
    tag_hierarchy: Vec<String>,
    /// タグ階層の最後のタグから取得する項目のリスト（重複しないこと）
    target_tag_value_names: Vec<TagValueName>,
}

impl XmlTagReadConfig {
    /// 新しい`XmlTagReadConfig`を作成します。
    ///
    /// # 引数
    /// - `tag_hierarchy`: タグ階層を表す`Vec<String>`。
    /// - `target_tag_value_names`: タグ階層の最後のタグから取得する項目のリスト（`HashSet<TagValueName>`）。
    ///
    /// # 戻り値
    /// - 新しい`XmlTagReadConfig`インスタンス
    ///
    pub fn new(tag_hierarchy: Vec<String>, target_tag_value_names: HashSet<TagValueName>) -> Self {
        XmlTagReadConfig {
            tag_hierarchy,
            target_tag_value_names: target_tag_value_names.into_iter().collect(),
        }
    }

    /// タグ階層を取得します。
    ///
    /// # 戻り値
    /// - タグ階層を表す`Vec<String>`
    pub fn get_tag_hierarchy(&self) -> Vec<String> {
        self.tag_hierarchy.clone()
    }

    /// タグの値を取得します。
    ///
    /// # 戻り値
    /// - タグ階層の最後のタグから取得する項目のリスト（`HashSet<TagValueName>`）
    pub fn get_target_tag_value_names(&self) -> HashSet<TagValueName> {
        let src_vec = self.target_tag_value_names.clone();
        let mut target_set = HashSet::new();
        for tag_value_name in src_vec {
            target_set.insert(tag_value_name.clone());
        }
        target_set
    }
}

/// `XmlReadConfig`は、XMLタグのグループ化設定を表します。
///
/// この構造体は、複数のタグ階層に関連付けられた読み取り設定を管理します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlReadConfig {
    /// タグ階層をキー、XMLタグ読み取り設定を値とするハッシュマップ
    tag_hierarchy_map: HashMap<Vec<String>, XmlTagReadConfig>,
    /// グループ化の単位となるタグ階層（タグ階層の一部であること）
    tag_group_hierarchy: Vec<String>,
}

impl XmlReadConfig {
    /// 新しい`XmlReadConfig`を作成します。
    ///
    /// # 引数
    /// - `tag_group_hierarchy`: グループ化の基準となるタグ階層
    ///
    /// # 戻り値
    /// - 新しい`XmlReadConfig`インスタンス
    pub fn new(tag_group_hierarchy: Vec<String>) -> Self {
        XmlReadConfig {
            tag_hierarchy_map: HashMap::new(),
            tag_group_hierarchy: tag_group_hierarchy.clone(),
        }
    }

    /// タグ階層をキー、XMLタグ読み取り設定を値とするハッシュマップを取得します。
    ///
    /// # 戻り値
    /// - タグ階層をキーとするハッシュマップ
    pub fn get_tag_hierarchy_map(&self) -> HashMap<Vec<String>, XmlTagReadConfig> {
        self.tag_hierarchy_map.clone()
    }

    /// グループ化の単位となるタグ階層を取得します。
    ///
    /// # 戻り値
    /// - グループ化の基準となるタグ階層（`Vec<String>`）
    pub fn get_tag_group_hierarchy(&self) -> Vec<String> {
        self.tag_group_hierarchy.clone()
    }

    /// XMLタグ読み取り設定をハッシュマップに挿入します。
    ///
    /// # 引数
    /// - `tag_read_config`: 挿入する`XmlTagReadConfig`
    ///
    /// # 戻り値
    /// - `Ok(())`: 正常に挿入された場合
    /// - `Err(String)`: エラーが発生した場合、エラーメッセージを含む
    pub fn insert_xml_tag_read_config_to_hash_map(
        &mut self,
        tag_read_config: &XmlTagReadConfig,
    ) -> Result<(), String> {
        let target = tag_read_config.clone();
        let tag_hierarchy = target.get_tag_hierarchy();
        if !Self::is_prefix(&self.get_tag_group_hierarchy(), &tag_hierarchy) {
            return Err(format!(
                "tag_group_hierarchy {:?} is not prefix of tag_hierarchy {:?}",
                tag_hierarchy,
                self.get_tag_group_hierarchy()
            ));
        }
        self.tag_hierarchy_map.insert(tag_hierarchy, target);
        Ok(())
    }

    /// タグ階層の一部であることを確認するためのヘルパー関数です。
    ///
    /// # 引数
    /// - `vec_a`: プレフィックスとして確認するベクター
    /// - `vec_b`: プレフィックスを持つベクター
    ///
    /// # 戻り値
    /// - `true`: `vec_a`が`vec_b`のプレフィックスである場合
    /// - `false`: `vec_a`が`vec_b`のプレフィックスでない場合
    fn is_prefix(vec_a: &[String], vec_b: &[String]) -> bool {
        vec_b.len() >= vec_a.len() && &vec_b[..vec_a.len()] == vec_a
    }
}
