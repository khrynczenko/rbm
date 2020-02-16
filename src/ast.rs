#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatementKind {
    Declaration,
    Expression,
    IfElse,
    For,
    Print,
    Return,
    Block,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionKind {
    Assignment,
    And,
    Or,
    Less,
    LessEqual,
    More,
    MoreEqual,
    Equal,
    NotEqual,
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulo,
    Power,
    Minus,
    Negation,
    Incrementation,
    Decrementation,
    Identifier,
    Literal,
    Array,
    Call,
    Subscript,
    FunctionCall,
    FunctionArg,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeKind {
    Void,
    Boolean,
    Character,
    Integer,
    Text,
    Array,
    Function,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub subtype: Option<Box<Type>>,
    pub param_list: Option<ParameterList>,
}

impl Type {
    pub fn from_name(name: String) -> Option<Type> {
        let type_ = match name.as_str() {
            "void"=> Type {kind: TypeKind::Void, subtype: None, param_list: None},
            "boolean"=> Type {kind: TypeKind::Boolean, subtype: None, param_list: None},
            "string"=> Type {kind: TypeKind::Text, subtype: None, param_list: None},
            "function"=> Type {kind: TypeKind::Function, subtype: None, param_list: None},
            "array"=> Type {kind: TypeKind::Array, subtype: None, param_list: None},
            "integer"=> Type {kind: TypeKind::Integer, subtype: None, param_list: None},
            "char"=> Type {kind: TypeKind::Character, subtype: None, param_list: None},
            _ => return None,
        };
        Some(type_)
    }

    pub fn attach_most_subtype(&mut self, subtype: Type) {
        match &mut self.subtype {
            None => self.subtype = Some(Box::new(subtype)),
            Some(nexter) => nexter.attach_most_subtype(subtype),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterList {
    pub name: String,
    pub kind: TypeKind,
    pub next: Option<Box<ParameterList>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub type_: Type,
    pub value: Option<Expression>,
    pub code: Option<Box<Statement>>,
    pub next: Option<Box<Declaration>>,
}

impl Declaration {
    pub fn new_value(name: String, type_: Type, value: Option<Expression>) -> Self {
        Declaration {
            name,
            type_,
            value,
            code: None,
            next: None,
        }
    }

    pub fn new_function(name: String, type_: Type, code: Option<Statement>) -> Self{
        Declaration {
            name,
            type_,
            value: None,
            code: code.map(|x| Box::new(x)),
            next: None,
        }
    }

    pub fn attach_most_next(&mut self, declaration: Declaration) {
        match &mut self.next {
            None => self.next = Some(Box::new(declaration)),
            Some(nexter) => nexter.attach_most_next(declaration),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub declaration: Option<Declaration>,
    pub expression: Option<Expression>,
    pub for_initial_expr: Option<Expression>,
    pub for_next_expr: Option<Expression>,
    pub body: Option<Box<Statement>>,
    pub else_body: Option<Box<Statement>>,
    pub next_statement: Option<Box<Statement>>,
}

impl Statement {
    pub fn new_declaration(declaration: Declaration) -> Self {
        Statement{kind: StatementKind::Declaration,
                  declaration: Some(declaration),
                  expression: None,
                  for_initial_expr: None,
                  for_next_expr: None,
                  body: None,
                  else_body: None,
                  next_statement: None}
    }

    pub fn new_expression(expression: Expression) -> Self {
        Statement{kind: StatementKind::Declaration,
                  declaration: None,
                  expression: Some(expression),
                  for_initial_expr: None,
                  for_next_expr: None,
                  body: None,
                  else_body: None,
                  next_statement: None}
    }

    pub fn new_for(initial_expression: Option<Expression>, condition: Option<Expression>, next_expression: Option<Expression>, body: Statement) -> Self {
        Statement{kind: StatementKind::For,
                  declaration: None,
                  expression: condition,
                  for_initial_expr: initial_expression,
                  for_next_expr: next_expression,
                  body: Some(Box::new(body)),
                  else_body: None,
                  next_statement: None}
    }

    pub fn new_if_else(condition: Expression, if_body: Statement, else_body: Option<Statement>) -> Self {
        Statement{kind: StatementKind::IfElse,
                  declaration: None,
                  expression: Some(condition),
                  for_initial_expr: None,
                  for_next_expr: None,
                  body: Some(Box::new(if_body)),
                  else_body: else_body.map(|x| Box::new(x)),
                  next_statement: None}
    }

    pub fn new_print(expression: Option<Expression>) -> Self {
        Statement{kind: StatementKind::Print,
                  declaration: None,
                  expression: expression,
                  for_initial_expr: None,
                  for_next_expr: None,
                  body: None,
                  else_body: None,
                  next_statement: None}
    }

    pub fn new_return(expression: Expression) -> Self {
        Statement{kind: StatementKind::Return,
                  declaration: None,
                  expression: Some(expression),
                  for_initial_expr: None,
                  for_next_expr: None,
                  body: None,
                  else_body: None,
                  next_statement: None}
    }
    pub fn new_block(block: Option<Statement>) -> Self {
        Statement{kind: StatementKind::Block,
                  declaration: None,
                  expression: None,
                  for_initial_expr: None,
                  for_next_expr: None,
                  body: None,
                  else_body: None,
                  next_statement: block.map(|x| Box::new(x))}
    }
    pub fn attach_most_next(&mut self, statement: Statement) {
        match &mut self.next_statement {
            None => self.next_statement = Some(Box::new(statement)),
            Some(next) => next.attach_most_next(statement),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionValue {
    Name(String),
    Integer(usize),
    Float(f64),
    Character(char),
    Text(String),
    Array(Vec<Expression>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub left: Option<Box<Expression>>,
    pub right: Option<Box<Expression>>,
    pub value: Option<ExpressionValue>,
}

impl Expression {
    pub fn new_value(value: ExpressionValue) -> Expression {
        let kind = match value {
            ExpressionValue::Name(_) => ExpressionKind::Identifier,
            _ => ExpressionKind::Literal,
        };
        let left = None;
        let right = None;
        Expression {
            kind,
            left,
            right,
            value: Some(value),
        }
    }
    pub fn attach_leftmost(&mut self, expr: Expression) {
        match &mut self.left {
            None => {
                self.left = Some(Box::new(expr));
            }
            Some(next_expr) => {
                next_expr.attach_leftmost(expr);
            }
        }
    }
    pub fn attach_rightmost(&mut self, expr: Expression) {
        match &mut self.right {
            None => {
                self.right = Some(Box::new(expr));
            }
            Some(next_expr) => {
                next_expr.attach_rightmost(expr);
            }
        }
    }
}
