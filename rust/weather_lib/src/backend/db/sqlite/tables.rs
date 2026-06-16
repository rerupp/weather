//! The Sqlite3 database table definitions, constants and helpers.

// the weather history database schema
pub mod weather;

// the cities database schema
pub mod cities;

/// Used internally to create an insert statement for a row in one of the tables.
///
/// # Arguments
///
/// * `table_name` is the name of the table.
/// * `column_params` provides the insert statement column names and named parameters.
///
fn create_insert_sql(table_name: &str, column_param: &[(&str, &str)]) -> String {
    let columns =
        column_param.iter().skip(1).map(|(column_name, _)| column_name.to_string()).collect::<Vec<_>>().join(",");
    let named_params =
        column_param.iter().skip(1).map(|(_, named_param)| named_param.to_string()).collect::<Vec<_>>().join(",");
    sql_query_builder::Insert::new()
        .insert_into(&format!("{table_name} ({columns})"))
        .values(&format!("({named_params})"))
        .to_string()
}

/// This trait is implemented by the table enumerations to provide a common API building SQL.
///
pub trait TblSqlBuilder {
    /// The variant column name.
    ///
    fn column(&self) -> &'static str;

    /// The variant named parameter.
    ///
    fn param(&self) -> &'static str;

    /// Generate the SQL fragment '`alias.column`'.
    ///
    /// # Arguments
    ///
    /// * `alias` is the alias name.
    ///
    fn alias_column(&self, alias: &str) -> String {
        format!("{}.{}", alias, self.column())
    }

    /// Generate the SQL fragment '`alias.column AS name`'.
    ///
    /// # Arguments
    ///
    /// * `alias` is the column alias.
    /// * `name` is the result set column name.
    ///
    fn alias_column_as_name(&self, alias: &str, name: &str) -> String {
        format!("{}.{} AS {}", alias, self.column(), name)
    }

    /// Generate the SQL fragment '`alias.column AS column`'.
    ///
    /// # Arguments
    ///
    /// * `alias` is the column alias.
    ///
    fn alias_column_as_column(&self, alias: &str) -> String {
        let column = self.column();
        format!("{}.{} AS {}", alias, column, column)
    }

    /// Generate the SQL fragment '`column=named_param`'.
    ///
    /// #Arguments
    ///
    /// * `param_opt` is the optional named parameter.
    ///
    // add where_param_named(...) if you need to set the name
    fn where_param(&self) -> String {
        format!("{}={}", self.column(), self.param())
    }

    /// Generate the SQL fragment '`alias.column=named_param`'.
    ///
    /// #Arguments
    ///
    /// * `alias` is the column
    ///
    // add alias_where_param_named(...) if you need to set the parameter name
    fn alias_where_param(&self, alias: &str) -> String {
        format!("{}.{}={}", alias, self.column(), self.param())
    }

    /// Generate the SQL fragment '`column ASC`'.
    ///
    fn column_asc(&self) -> String {
        format!("{} ASC", self.column())
    }

    /// Generate the SQL fragment '`COUNT(alias.column)=name`'.
    ///
    /// #Arguments
    ///
    /// * `alias` is the column alias
    /// * `name` is the result set column name.
    ///
    fn alias_count_as(&self, alias: &str, name: &str) -> String {
        format!("COUNT({}) AS {}", self.alias_column(alias), name)
    }

    /// Generate the SQL fragment '`SUM(alias.column)=name`'.
    ///
    /// #Arguments
    ///
    /// * `alias` is the column alias
    /// * `name` is the result set column name.
    ///
    fn alias_sum_as(&self, alias: &str, name: &str) -> String {
        format!("SUM({}) AS {}", self.alias_column(alias), name)
    }

    /// Generate the SQL fragment '`SUM(alias.column)=column`'.
    ///
    /// #Arguments
    ///
    /// * `alias` is the column alias
    ///
    fn alias_sum_as_column(&self, alias: &str) -> String {
        format!("SUM({}) AS {}", self.alias_column(alias), self.column())
    }
}

/// Generate a `(param, &(value) as &dyn ::rusqlite::ToSql)` tuple.
macro_rules! named_param {
    ($column: path, $value: expr) => {
        ($column.param(), &($value) as &dyn ::rusqlite::ToSql)
    };
}
pub(in crate::backend::db::sqlite) use named_param;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builders() {
        enum Testcase {
            One,
        }
        impl Testcase {
            const COLUMN_PARAM: [(&str, &str); 1] = [("column", ":column")];
        }
        impl TblSqlBuilder for Testcase {
            // fn table(&self) -> &'static str {
            //     "table"
            // }
            fn column(&self) -> &'static str {
                Self::COLUMN_PARAM[Self::One as usize].0
            }
            fn param(&self) -> &'static str {
                Self::COLUMN_PARAM[Self::One as usize].1
            }
        }
        // column and named parameters
        assert_eq!(Testcase::One.column(), "column");
        assert_eq!(Testcase::One.param(), ":column");

        // builders
        assert_eq!(Testcase::One.column_asc(), "column ASC");
        assert_eq!(Testcase::One.alias_column("alias"), "alias.column");
        assert_eq!(Testcase::One.alias_column_as_column("alias"), "alias.column AS column");
        assert_eq!(Testcase::One.where_param(), "column=:column");
        // assert_eq!(Testcase::One.where_param(Some(":name")), "column=:name");
        assert_eq!(Testcase::One.alias_where_param("alias"), "alias.column=:column");
        // assert_eq!(Testcase::One.alias_where_param("alias", Some(":param")), "alias.column=:param");
        assert_eq!(Testcase::One.alias_count_as("alias", "name"), "COUNT(alias.column) AS name");
    }
}
