use std::ops::Range;

use crate::{
    ForeignKeyId, ForeignKeyWalker, IndexColumnWalker, IndexId, IndexWalker, NamespaceId, Table, TableColumnId,
    TableColumnWalker, TableId, Walker,
};

/// Traverse a table.
pub type TableWalker<'a> = Walker<'a, TableId>;

impl<'a> TableWalker<'a> {
    /// Get a column in the table, by name.
    pub fn column(self, column_name: &str) -> Option<TableColumnWalker<'a>> {
        self.columns().find(|column| column.name() == column_name)
    }

    /// Get a column in the table, by name.
    pub fn column_case_insensitive(self, column_name: &str) -> Option<TableColumnWalker<'a>> {
        self.columns().find(|column| column.name() == column_name)
    }

    fn columns_range(self) -> Range<usize> {
        super::range_for_key(&self.schema.table_columns, self.id, |(tid, _)| *tid)
    }

    /// Traverse the table's columns.
    pub fn columns(self) -> impl ExactSizeIterator<Item = TableColumnWalker<'a>> {
        self.columns_range()
            .into_iter()
            .map(move |idx| self.walk(TableColumnId(idx as u32)))
    }

    /// The number of foreign key constraints on the table.
    pub fn foreign_key_count(self) -> usize {
        self.foreign_keys_range().into_iter().len()
    }

    /// Traverse the indexes on the table.
    pub fn indexes(self) -> impl ExactSizeIterator<Item = IndexWalker<'a>> {
        let range = super::range_for_key(&self.schema.indexes, self.id, |idx| idx.table_id);
        range.map(move |idx| self.walk(IndexId(idx as u32)))
    }

    /// Traverse the foreign keys on the table.
    pub fn foreign_keys(self) -> impl ExactSizeIterator<Item = ForeignKeyWalker<'a>> {
        self.foreign_keys_range()
            .map(move |id| self.walk(ForeignKeyId(id as u32)))
    }

    /// Traverse foreign keys from other tables, referencing current table.
    pub fn referencing_foreign_keys(self) -> impl Iterator<Item = ForeignKeyWalker<'a>> {
        self.schema
            .table_walkers()
            .filter(move |t| t.id != self.id)
            .flat_map(|t| t.foreign_keys())
            .filter(move |fk| fk.referenced_table().id == self.id)
    }

    /// The table name.
    pub fn name(self) -> &'a str {
        &self.table().name
    }

    fn foreign_keys_range(self) -> Range<usize> {
        super::range_for_key(&self.schema.foreign_keys, self.id, |fk| fk.constrained_table)
    }

    /// Try to traverse a foreign key for a single column.
    pub fn foreign_key_for_column(self, column: TableColumnId) -> Option<ForeignKeyWalker<'a>> {
        self.foreign_keys().find(|fk| {
            let cols = fk.columns();
            cols.len() == 1 && cols[0].constrained_column == column
        })
    }

    /// The namespace the table belongs to, if defined.
    pub fn namespace(self) -> Option<&'a str> {
        self.schema
            .namespaces
            .get(self.table().namespace_id.0 as usize)
            .map(|s| s.as_str())
    }

    /// The namespace the table belongs to.
    pub fn namespace_id(self) -> NamespaceId {
        self.table().namespace_id
    }

    /// Traverse to the primary key of the table.
    pub fn primary_key(self) -> Option<IndexWalker<'a>> {
        self.indexes().find(|idx| idx.is_primary_key())
    }

    /// The columns that are part of the primary keys.
    pub fn primary_key_columns(self) -> Option<impl ExactSizeIterator<Item = IndexColumnWalker<'a>>> {
        self.primary_key().map(|pk| pk.columns())
    }

    /// How many columns are in the primary key? Returns 0 in the absence of a pk.
    pub fn primary_key_columns_count(self) -> usize {
        self.primary_key_columns().map(|cols| cols.len()).unwrap_or(0)
    }

    /// Is the table a partition table?
    pub fn is_partition(self) -> bool {
        self.table().is_partition
    }

    /// Reference to the underlying `Table` struct.
    fn table(self) -> &'a Table {
        &self.schema.tables[self.id.0 as usize]
    }
}
