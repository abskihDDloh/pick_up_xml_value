use std::collections::HashMap;

use super::output_tag_value::OutPutTagValue;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::reader) struct TagHierarchyStore {
    tag_store: HashMap<Vec<String>, OutPutTagValue>,
}
impl TagHierarchyStore {
    pub(in crate::reader) fn new() -> Self {
        TagHierarchyStore {
            tag_store: HashMap::new(),
        }
    }
    pub(in crate::reader) fn get(&self, key: &Vec<String>) -> Option<&OutPutTagValue> {
        self.tag_store.get(key)
    }
    pub(in crate::reader) fn remove(&mut self, key: &Vec<String>) -> Option<OutPutTagValue> {
        self.tag_store.remove(key)
    }

    /// 指定されたキーに対応する値への可変参照を取得します。
    ///
    /// # 引数
    /// - `key`: 検索対象のキーとなる `Vec<String>`。
    ///
    /// # 戻り値
    /// - キーが存在する場合は対応する `OutPutTagValue` への可変参照を返します。
    /// - キーが存在しない場合は、新しい `OutPutTagValue` を作成し、`key` をタグ階層として設定した上で挿入し、その参照を返します。
    ///
    /// # 動作
    /// - キーが `tag_store` に存在する場合、その値への可変参照を返します。
    /// - キーが存在しない場合は、`OutPutTagValue::default()` を使用して新しい値を作成し、`set_tag_hitrarchy()` を呼び出してタグ階層を設定した後、`tag_store` に挿入します。
    ///
    /// # 使用例
    /// ```
    /// let mut store = TagHierarchyStore::new();
    /// let key = vec!["level1".to_string(), "level2".to_string()];
    ///
    /// // キーが存在しない場合、新しい値を作成して挿入
    /// let value = store.get_mut(&key);
    /// assert!(value.is_some());
    ///
    /// // キーが存在する場合、既存の値への参照を取得
    /// let existing_value = store.get_mut(&key);
    /// assert!(existing_value.is_some());
    /// ```
    pub(in crate::reader) fn get_mut(&mut self, key: &Vec<String>) -> Option<&mut OutPutTagValue> {
        if self.tag_store.contains_key(key) {
            return self.tag_store.get_mut(key);
        } else {
            let mut out_put_tag_value = OutPutTagValue::default();
            let _ = out_put_tag_value.set_tag_hierarchy(key);
            self.tag_store.insert(key.clone(), out_put_tag_value);
        }
        self.tag_store.get_mut(key)
    }
}
