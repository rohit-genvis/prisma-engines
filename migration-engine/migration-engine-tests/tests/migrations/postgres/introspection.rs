use migration_engine_tests::test_api::*;

#[test]
fn introspect_partition_tables() {
    // Postgres9 does not support partition tables, and Postgres10 does not support primary keys on
    // partition tables.
    let test_db = test_setup::only!(Postgres11, Postgres12, Postgres13, Postgres14, Postgres15 ; exclude: CockroachDb);
    let (_, url_str) = tok(test_setup::postgres::create_postgres_database(
        test_db.url(),
        "postgres_introspect_partition_tables",
    ))
    .unwrap();

    let me = migration_core::migration_api(None, None).unwrap();

    let script = r#"
CREATE TABLE IF NOT EXISTS blocks
(
    id int NOT NULL,
    account text COLLATE pg_catalog."default" NOT NULL,
    block_source_id int,
    CONSTRAINT blocks_pkey PRIMARY KEY (account, id)
) PARTITION BY RANGE (id);


CREATE TABLE blocks_p1_0 PARTITION OF blocks
    FOR VALUES FROM (0) TO (1000);

CREATE TABLE blocks_p2_0 PARTITION OF blocks
    FOR VALUES FROM (1001) TO (2000);

ALTER TABLE blocks
      ADD CONSTRAINT block_source_block_fk FOREIGN KEY (block_source_id, account)
        REFERENCES blocks (id, account) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE CASCADE; "#;

    tok(me.db_execute(DbExecuteParams {
        datasource_type: DbExecuteDatasourceType::Url(UrlContainer { url: url_str.clone() }),
        script: script.to_owned(),
    }))
    .unwrap();

    let schema = format! {
        r#"
            datasource db {{
                provider = "postgres"
                url = "{url_str}"
            }}
        "#,
    };

    let result = tok(me.introspect(migration_core::json_rpc::types::IntrospectParams {
        composite_type_depth: -1,
        force: false,
        schema,
        schemas: None,
    }))
    .unwrap();

    let expected = format!(
        r#"datasource db {{
  provider = "postgres"
  url      = "{}"
}}

/// This table is a partition table and requires additional setup for migrations. Visit https://pris.ly/d/partition-tables for more info.
model blocks {{
  id              Int
  account         String
  block_source_id Int?
  blocks          blocks?  @relation("blocksToblocks", fields: [block_source_id, account], references: [id, account], onDelete: Cascade, onUpdate: NoAction, map: "block_source_block_fk")
  other_blocks    blocks[] @relation("blocksToblocks")

  @@id([account, id])
}}
"#,
        url_str
    );
    pretty_assertions::assert_eq!(expected, result.datamodel.as_str());
}
