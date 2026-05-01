//! This module builds the Cities query from the [CityFilter]

use super::super::sql_match_condition;
use super::persistence::{city, country, region};
use crate::prelude::LocationFilter;
use sql_query_builder as sql;

/// Create a SQL table column description that includes an alias name
/// # Params
/// * `table` is the table name.
/// * `column` is the table column
/// * `alias` is the query result column name.
///
macro_rules! column_with_alias {
    ($table:expr, $column:expr, $alias: expr) => {
        &format!("{t}.{c} AS {a}", t = $table, c = $column, a = $alias)
    };
}

/// Create the description of a SQL table join.
/// # Params
/// * `lhs` is the left hand side table name.
/// * `lhs_col` is the left hand side table column.
/// * `rhs` is the right hand side table name.
/// * `rhs_col` is the right hand side table column.
///
macro_rules! table_join {
    ($lhs: expr, $lhs_col: expr, $rhs: expr, $rhs_col: expr) => {
        &format!("{l} ON {l}.{la} = {r}.{ra}", l = $lhs, la = $lhs_col, r = $rhs, ra = $rhs_col)
    };
}

/// The query result row column alias for the country name.
pub const COUNTRY_NAME: &str = "country_name";

/// The query result row column alias for the country code.
pub const COUNTRY_CODE: &str = "country_code";

/// The query result row column alias for the region name.
pub const REGION_NAME: &str = "region_name";

/// The query result row column alias for the region code.
pub const REGION_CODE: &str = "region_code";

/// The query result row column alias for the city name.
pub const CITY_NAME: &str = "city_name";

/// The query result row column alias for the latitude.
pub const LATITUDE: &str = "lat";

/// The query result row column alias for the longitude.
pub const LONGITUDE: &str = "lng";

/// The query result row column alias for the timezone.
pub const TIMEZONE: &str = "tz";

/// The query result row column alias for the region city count.
pub const CITY_COUNT: &str = "city_count";

/// Generate the boilerplate SQL common to the queries.
///
macro_rules! query {
    () => {
        sql::Select::new()
            .select(column_with_alias!(country::TABLE, "name", COUNTRY_NAME))
            .select(column_with_alias!(country::TABLE, "code", COUNTRY_CODE))
            .select(column_with_alias!(region::TABLE, "name", REGION_NAME))
            .select(column_with_alias!(region::TABLE, "code", REGION_CODE))
            .from(city::TABLE)
            .inner_join(table_join!(region::TABLE, "id", city::TABLE, "rid"))
            .inner_join(table_join!(country::TABLE, "id", region::TABLE, "coid"))
    };
}

/// Create the query used to find the cities. The results of the query are
/// ordered by city name, region name, and country name.
///
/// # Arguments
///
/// * `filters` is used to find cities.
/// * `limit` restricts the number of cities returned.
///
pub fn city(filters: Option<Vec<LocationFilter>>, limit: usize) -> crate::Result<String> {
    let mut sql = query!()
        .select(column_with_alias!(city::TABLE, "name", CITY_NAME))
        .select(column_with_alias!(city::TABLE, "lat", LATITUDE))
        .select(column_with_alias!(city::TABLE, "lng", LONGITUDE))
        .select(column_with_alias!(city::TABLE, "tz", TIMEZONE))
        .order_by(CITY_NAME)
        .order_by(REGION_NAME)
        .order_by(COUNTRY_NAME)
        .limit(&limit.to_string());

    if let Some(filters) = filters {
        for filter in filters {
            let mut filter_sql = vec![];
            if let Some(city_filter) = &filter.city {
                let name_match = sql_match_condition(CITY_NAME, city_filter)?;
                filter_sql.push(name_match);
            }
            if let Some(region_filter) = &filter.region {
                let region_name_match = sql_match_condition(REGION_NAME, region_filter)?;
                let region_code_match = sql_match_condition(REGION_CODE, region_filter)?;
                filter_sql.push(format!("({region_name_match} OR {region_code_match})"));
            }
            if let Some(country_filter) = &filter.country {
                let country_name_match = sql_match_condition(COUNTRY_NAME, country_filter)?;
                let country_code_match = sql_match_condition(COUNTRY_CODE, country_filter)?;
                filter_sql.push(format!("({country_name_match} OR {country_code_match})"));
            }
            if filter_sql.len() > 0 {
                sql = sql.where_or(format!("({})", filter_sql.join(" AND ")).as_str());
            }
        }
    }
    Ok(sql.as_string())
}

/// Create the query used to get information about the Cities database. The results
/// are orders by country name and country region.
pub fn details() -> String {
    query!()
        .select(&format!("COUNT(*) AS {CITY_COUNT}"))
        .group_by(COUNTRY_NAME)
        .group_by(REGION_NAME)
        .order_by(COUNTRY_NAME)
        .order_by(REGION_NAME)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city() {
        macro_rules! filter {
            ($name: literal, $region: literal, $country: literal) => {{
                let mut filter = LocationFilter::default();
                if $name.len() > 0 {
                    filter.city.replace($name.to_string());
                }
                if $region.len() > 0 {
                    filter.region.replace($region.to_string());
                }
                if $country.len() > 0 {
                    filter.country.replace($country.to_string());
                }
                filter
            }};
        }

        let testcase = |filters, limit| super::city(filters, limit).unwrap();

        let sql = testcase(None, 50);
        assert!(sql.contains("country.name AS country_name"));
        assert!(sql.contains("country.code AS country_code"));
        assert!(sql.contains("region.name AS region_name"));
        assert!(sql.contains("region.code AS region_code"));
        assert!(sql.contains("city.name AS city_name"));
        assert!(sql.contains("city.lat AS lat"));
        assert!(sql.contains("city.lng AS lng"));
        assert!(sql.contains("city.tz AS tz"));
        assert!(sql.contains("FROM city"));
        assert!(sql.contains("INNER JOIN region ON region.id = city.rid"));
        assert!(sql.contains("INNER JOIN country ON country.id = region.coid"));
        assert!(!sql.contains("WHERE"));
        assert!(sql.contains("LIMIT 50"));
        assert!(sql.contains("ORDER BY city_name, region_name, country_name"));

        let sql = testcase(Some(vec![filter!("city", "", "")]), 35);
        assert!(sql.contains("LIMIT 35"));
        assert!(sql.contains("WHERE (city_name = 'city')"));

        let sql = testcase(Some(vec![filter!("", "region", "")]), 15);
        assert!(sql.contains("LIMIT 15"));
        assert!(sql.contains("WHERE ((region_name = 'region' OR region_code = 'region'))"));

        let sql = testcase(Some(vec![filter!("", "", "country")]), 25);
        assert!(sql.contains("LIMIT 25"));
        assert!(sql.contains("WHERE ((country_name = 'country' OR country_code = 'country'))"));

        let sql = testcase(Some(vec![filter!("city", "region", "country")]), 135);
        assert!(sql.contains("LIMIT 135"));
        assert!(sql.contains(
            format!(
                "WHERE ({} AND {} AND {})",
                "city_name = 'city'",
                "(region_name = 'region' OR region_code = 'region')",
                "(country_name = 'country' OR country_code = 'country')"
            )
            .as_str()
        ));

        let sql = testcase(Some(vec![filter!("city1", "", ""), filter!("city2", "", "")]), 10);
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("WHERE (city_name = 'city1')"));
        assert!(sql.contains("OR (city_name = 'city2')"));
    }

    #[test]
    fn city_details() {
        let sql = details();
        assert!(sql.contains("country.name AS country_name"));
        assert!(sql.contains("country.code AS country_code"));
        assert!(sql.contains("region.name AS region_name"));
        assert!(sql.contains("region.code AS region_code"));
        assert!(sql.contains("COUNT(*) AS city_count"));
        assert!(sql.contains("FROM city"));
        assert!(sql.contains("INNER JOIN region ON region.id = city.rid"));
        assert!(sql.contains("INNER JOIN country ON country.id = region.coid"));
        assert!(sql.contains("GROUP BY country_name, region_name"));
        assert!(sql.contains("ORDER BY country_name, region_name"))
    }
}
