use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use tantivy::query::QueryParser;
use tantivy::{self, schema, Index, ReloadPolicy};

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
  P: AsRef<Path>,
{
  let file = File::open(filename)?;
  Ok(io::BufReader::new(file).lines())
}

const CHAR_FILE: &str = "characters.txt";

#[derive(Clone)]
pub struct SearchEngine {
  id: i32,
  pub index: Index,
  pub query_parser: QueryParser,
  pub reader: tantivy::IndexReader,
  pub name_field: schema::Field,
  pub char_field: schema::Field,
}

impl druid::Data for SearchEngine {
  fn same(&self, other: &Self) -> bool {
    // std::ptr::eq(&self, &other)
    // self as *const Self == other as *const Self
    // true
    self.id == other.id
  }
}

pub fn new_query_parser() -> tantivy::Result<SearchEngine> {
  let mut schema_builder = schema::Schema::builder();
  schema_builder.add_text_field("name", schema::TEXT | schema::STORED);
  schema_builder.add_text_field("char", schema::STORED);
  let schema = schema_builder.build();
  let index = Index::create_in_ram(schema.clone());
  let mut index_writer = index.writer(50_000_000)?;
  let name_field = schema.get_field("name").unwrap();
  let char_field = schema.get_field("char").unwrap();

  let lines = read_lines(CHAR_FILE)?;
  for line in lines.flatten() {
    let name: &str = &line[line.char_indices().nth(2).unwrap().0..];
    if let Some(c) = line.chars().nth(0) {
      index_writer.add_document(tantivy::doc!(
        name_field => name,
        char_field => c.to_string(),
      ));
    }
  }

  index_writer.commit()?;

  let reader = index
    .reader_builder()
    .reload_policy(ReloadPolicy::OnCommit)
    .try_into()?;
  let searcher = reader.searcher();

  let query_parser = QueryParser::for_index(&index, vec![name_field]);
  Ok(SearchEngine {
    id: rand::random(),
    index: index,
    query_parser: query_parser,
    reader: reader,
    name_field: name_field,
    char_field: char_field,
  })
}
