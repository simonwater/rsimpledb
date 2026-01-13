use crate::parse::{
    BadSyntaxException, CreateIndexData, CreateTableData, CreateViewData, DeleteData, InsertData,
    Lexer, ModifyData, QueryData,
};
use crate::query::{Constant, Expression, Predicate, Term};
use crate::record::Schema;

pub struct Parser {
    lex: Lexer,
}

impl Parser {
    pub fn new(s: &str) -> Self {
        Parser { lex: Lexer::new(s) }
    }

    pub fn field(&mut self) -> String {
        self.lex.eat_id()
    }

    pub fn constant(&mut self) -> Constant {
        if self.lex.match_string_constant() {
            Constant::from_string(self.lex.eat_string_constant())
        } else {
            Constant::from_int(self.lex.eat_int_constant())
        }
    }

    pub fn expression(&mut self) -> Expression {
        if self.lex.match_id() {
            Expression::from_field(self.field())
        } else {
            Expression::from_constant(self.constant())
        }
    }

    pub fn term(&mut self) -> Term {
        let lhs = self.expression();
        self.lex.eat_delim('=');
        let rhs = self.expression();
        Term::new(lhs, rhs)
    }

    pub fn predicate(&mut self) -> Predicate {
        let mut pred = Predicate::from_term(self.term());
        if self.lex.match_keyword("and") {
            self.lex.eat_keyword("and");
            pred.conjoin_with(self.predicate());
        }
        pred
    }

    pub fn query(&mut self) -> QueryData {
        self.lex.eat_keyword("select");
        let fields = self.select_list();
        self.lex.eat_keyword("from");
        let tables = self.table_list();
        let mut pred = Predicate::new();
        if self.lex.match_keyword("where") {
            self.lex.eat_keyword("where");
            pred = self.predicate();
        }
        QueryData::new(fields, tables, pred)
    }

    /// Methods for parsing the various update commands
    pub fn update_cmd(&mut self) -> UpdateCommand {
        if self.lex.match_keyword("insert") {
            UpdateCommand::Insert(self.insert())
        } else if self.lex.match_keyword("delete") {
            UpdateCommand::Delete(self.delete())
        } else if self.lex.match_keyword("update") {
            UpdateCommand::Modify(self.modify())
        } else {
            UpdateCommand::Create(self.create())
        }
    }

    fn create(&mut self) -> CreateCommand {
        self.lex.eat_keyword("create");
        if self.lex.match_keyword("table") {
            CreateCommand::Table(self.create_table())
        } else if self.lex.match_keyword("view") {
            CreateCommand::View(self.create_view())
        } else {
            CreateCommand::Index(self.create_index())
        }
    }

    pub fn delete(&mut self) -> DeleteData {
        self.lex.eat_keyword("delete");
        self.lex.eat_keyword("from");
        let tblname = self.lex.eat_id();
        let mut pred = Predicate::new();
        if self.lex.match_keyword("where") {
            self.lex.eat_keyword("where");
            pred = self.predicate();
        }
        DeleteData::new(tblname, pred)
    }

    pub fn insert(&mut self) -> InsertData {
        self.lex.eat_keyword("insert");
        self.lex.eat_keyword("into");
        let tblname = self.lex.eat_id();
        self.lex.eat_delim('(');
        let flds = self.field_list();
        self.lex.eat_delim(')');
        self.lex.eat_keyword("values");
        self.lex.eat_delim('(');
        let vals = self.const_list();
        self.lex.eat_delim(')');
        InsertData::new(tblname, flds, vals)
    }

    fn field_list(&mut self) -> Vec<String> {
        let mut list = Vec::new();
        list.push(self.field());
        while self.lex.match_delim(',') {
            self.lex.eat_delim(',');
            list.push(self.field());
        }
        list
    }

    fn const_list(&mut self) -> Vec<Constant> {
        let mut list = Vec::new();
        list.push(self.constant());
        while self.lex.match_delim(',') {
            self.lex.eat_delim(',');
            list.push(self.constant());
        }
        list
    }

    pub fn modify(&mut self) -> ModifyData {
        self.lex.eat_keyword("update");
        let tblname = self.lex.eat_id();
        self.lex.eat_keyword("set");
        let fldname = self.field();
        self.lex.eat_delim('=');
        let newval = self.expression();
        let mut pred = Predicate::new();
        if self.lex.match_keyword("where") {
            self.lex.eat_keyword("where");
            pred = self.predicate();
        }
        ModifyData::new(tblname, fldname, newval, pred)
    }

    pub fn create_table(&mut self) -> CreateTableData {
        self.lex.eat_keyword("table");
        let tblname = self.lex.eat_id();
        self.lex.eat_delim('(');
        let sch = self.field_defs();
        self.lex.eat_delim(')');
        CreateTableData::new(tblname, sch)
    }

    fn field_defs(&mut self) -> Schema {
        let mut schema = self.field_def();
        while self.lex.match_delim(',') {
            self.lex.eat_delim(',');
            let schema2 = self.field_defs();
            schema.add_all(&schema2);
        }
        schema
    }

    fn field_def(&mut self) -> Schema {
        let fldname = self.field();
        self.field_type(fldname)
    }

    fn field_type(&mut self, fldname: String) -> Schema {
        let mut schema = Schema::new();
        if self.lex.match_keyword("int") {
            self.lex.eat_keyword("int");
            schema.add_int_field(&fldname);
        } else {
            self.lex.eat_keyword("varchar");
            self.lex.eat_delim('(');
            let str_len = self.lex.eat_int_constant();
            self.lex.eat_delim(')');
            schema.add_string_field(&fldname, str_len);
        }
        schema
    }

    pub fn create_view(&mut self) -> CreateViewData {
        self.lex.eat_keyword("view");
        let viewname = self.lex.eat_id();
        self.lex.eat_keyword("as");
        let qd = self.query();
        CreateViewData::new(viewname, qd)
    }

    pub fn create_index(&mut self) -> CreateIndexData {
        self.lex.eat_keyword("index");
        let idxname = self.lex.eat_id();
        self.lex.eat_keyword("on");
        let tblname = self.lex.eat_id();
        self.lex.eat_delim('(');
        let fldname = self.field();
        self.lex.eat_delim(')');
        CreateIndexData::new(idxname, tblname, fldname)
    }

    fn select_list(&mut self) -> Vec<String> {
        let mut list = Vec::new();
        list.push(self.field());
        while self.lex.match_delim(',') {
            self.lex.eat_delim(',');
            list.push(self.field());
        }
        list
    }

    fn table_list(&mut self) -> std::collections::HashSet<String> {
        let mut set = std::collections::HashSet::new();
        set.insert(self.lex.eat_id());
        while self.lex.match_delim(',') {
            self.lex.eat_delim(',');
            set.insert(self.lex.eat_id());
        }
        set
    }
}

/// Enum to represent different update commands
pub enum UpdateCommand {
    Insert(InsertData),
    Delete(DeleteData),
    Modify(ModifyData),
    Create(CreateCommand),
}

/// Enum to represent different create commands
pub enum CreateCommand {
    Table(CreateTableData),
    View(CreateViewData),
    Index(CreateIndexData),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_test() {
        let s = "select name, address from t1 where id = 123 and 18 = age";
        let mut parser = Parser::new(s);
        let q_data = parser.query();
        assert_eq!(s, format!("{}", q_data));

        let s = "update t1 set name = 'abc' where id = 123 and 18 = age";
        let mut parser = Parser::new(s);
        parser.update_cmd();
    }
}
