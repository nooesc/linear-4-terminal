use chrono::{Duration, Utc};
use regex::Regex;

use super::builder::{
    FilterBuilder, FilterCondition, FilterExpression, FilterField, FilterGroup, FilterOperator, FilterValue,
    LogicalOperator,
};

/// Token types for the filter parser
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Field(String),
    Operator(String),
    Value(String),
    And,
    Or,
    Not,
    LeftParen,
    RightParen,
    Colon,
    Comma,
}

/// UTF-8 safe tokenizer for filter queries
struct Tokenizer {
    chars: Vec<char>,
    position: usize,
    last_operator: Option<String>,
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            position: 0,
            last_operator: None,
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();

        while self.position < self.chars.len() {
            self.skip_whitespace();

            if self.position >= self.chars.len() {
                break;
            }

            let token = if let Some(token) = self.try_parse_operator() {
                Some(token)
            } else if let Some(token) = self.try_parse_keyword() {
                Some(token)
            } else if let Some(token) = self.try_parse_special() {
                Some(token)
            } else if let Some(token) = self.try_parse_value()? {
                Some(token)
            } else {
                return Err(ParseError::UnexpectedCharacter {
                    position: self.position,
                    char: self.current_char().unwrap_or(' '),
                });
            };

            if let Some(token) = token {
                tokens.push(token);
            } else {
                return Err(ParseError::UnexpectedCharacter {
                    position: self.position,
                    char: self.current_char().unwrap_or(' '),
                });
            }
        }

        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.position += 1;
            } else {
                break;
            }
        }
    }

    fn current_char(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
        self.chars.get(self.position + offset).copied()
    }

    #[allow(dead_code)]
    fn peek_literal(&self, literal: &str) -> bool {
        let mut index = 0;
        for expected in literal.chars() {
            if self.chars.get(self.position + index).copied() != Some(expected) {
                return false;
            }
            index += 1;
        }

        true
    }

    fn starts_with_keyword(&self, keyword: &str) -> bool {
        let keyword_len = keyword.chars().count();
        let left_boundary = self.position == 0 || !self.is_word_char(self.chars.get(self.position - 1).copied());
        let right_boundary = if keyword.ends_with(':') {
            true
        } else {
            self.is_word_boundary_at(self.position + keyword_len)
        };

        left_boundary
            && right_boundary
            && self.starts_with_case_insensitive(keyword)
    }

    fn starts_with_case_insensitive(&self, expected: &str) -> bool {
        let mut index = 0;
        for expected_char in expected.chars() {
            if self.chars.get(self.position + index).copied().unwrap_or('\0').to_ascii_lowercase()
                != expected_char.to_ascii_lowercase()
            {
                return false;
            }

            index += 1;
        }

        true
    }

    fn is_word_char(&self, c: Option<char>) -> bool {
        c.map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false)
    }

    fn is_word_boundary_at(&self, index: usize) -> bool {
        if index >= self.chars.len() {
            true
        } else {
            !self.is_word_char(self.chars.get(index).copied())
        }
    }

    fn try_parse_operator(&mut self) -> Option<Token> {
        let op = match (self.current_char(), self.peek_char(1)) {
            (Some('!'), Some('=')) => Some("!="),
            (Some('!'), Some('~')) => Some("!~"),
            (Some('>'), Some('=')) => Some(">="),
            (Some('<'), Some('=')) => Some("<="),
            (Some('~'), Some('=')) => Some("~="),
            (Some('^'), Some('=')) => Some("^="),
            (Some('$'), Some('=')) => Some("$="),
            _ => None,
        };

        if let Some(operator) = op {
            self.position += 2;
            self.last_operator = Some(operator.to_string());
            return Some(Token::Operator(operator.to_string()));
        }

        match self.current_char()? {
            '=' => {
                self.position += 1;
                self.last_operator = Some("=".to_string());
                Some(Token::Operator("=".to_string()))
            }
            '>' => {
                self.position += 1;
                self.last_operator = Some(">".to_string());
                Some(Token::Operator(">".to_string()))
            }
            '<' => {
                self.position += 1;
                self.last_operator = Some("<".to_string());
                Some(Token::Operator("<".to_string()))
            }
            '~' => {
                self.position += 1;
                self.last_operator = Some("~".to_string());
                Some(Token::Operator("~".to_string()))
            }
            '^' => {
                self.position += 1;
                self.last_operator = Some("^".to_string());
                Some(Token::Operator("^".to_string()))
            }
            '$' => {
                self.position += 1;
                self.last_operator = Some("$".to_string());
                Some(Token::Operator("$".to_string()))
            }
            '!' => {
                self.position += 1;
                self.last_operator = Some("!".to_string());
                Some(Token::Operator("!".to_string()))
            }
            _ => None,
        }
    }

    fn try_parse_keyword(&mut self) -> Option<Token> {
        if self.starts_with_keyword("and") {
            self.position += 3;
            return Some(Token::And);
        }

        if self.starts_with_keyword("or") {
            self.position += 2;
            return Some(Token::Or);
        }

        if self.starts_with_keyword("not") {
            self.position += 3;
            return Some(Token::Not);
        }

        if self.starts_with_keyword("in:") {
            self.position += 3;
            self.last_operator = Some("in".to_string());
            return Some(Token::Operator("in".to_string()));
        }

        if self.starts_with_keyword("has:") {
            self.position += 4;
            return Some(Token::Operator("has".to_string()));
        }

        None
    }

    fn try_parse_special(&mut self) -> Option<Token> {
        match self.current_char()? {
            ':' => {
                self.position += 1;
                Some(Token::Colon)
            }
            '(' => {
                self.position += 1;
                Some(Token::LeftParen)
            }
            ')' => {
                self.position += 1;
                Some(Token::RightParen)
            }
            ',' => {
                self.position += 1;
                Some(Token::Comma)
            }
            _ => None,
        }
    }

    fn try_parse_value(&mut self) -> Result<Option<Token>, ParseError> {
        if self.current_char() != Some('"') {
            return Ok(self.parse_unquoted_value());
        }

        self.position += 1;
        let mut value = String::new();

        while let Some(c) = self.current_char() {
            if c == '"' {
                self.position += 1;
                return Ok(Some(Token::Value(value)));
            }

            if c == '\\' {
                self.position += 1;

                match self.current_char() {
                    Some(next) => {
                        value.push(next);
                        self.position += 1;
                    }
                    None => {
                        return Err(ParseError::UnterminatedString {
                            position: self.position,
                        })
                    }
                }

                continue;
            }

            value.push(c);
            self.position += 1;
        }

        Err(ParseError::UnterminatedString {
            position: self.position,
        })
    }

    fn parse_unquoted_value(&mut self) -> Option<Token> {
        let start = self.position;
        let include_commas = self.last_operator.as_deref() == Some("in");

        let mut position = self.position;
        while position < self.chars.len() {
            let c = self.chars[position];
            let next = self.chars.get(position + 1).copied();

            if c.is_whitespace() || c == ':' || c == '(' || c == ')' || (!include_commas && c == ',') {
                break;
            }

            if c == '!'
                || c == '='
                || c == '>'
                || c == '<'
                || c == '~'
                || c == '^'
                || c == '$'
                || (c == '!' && (next == Some('=') || next == Some('~')))
                || (c == '~' && next == Some('='))
                || (c == '^' && next == Some('='))
                || (c == '$' && next == Some('='))
            {
                break;
            }

            position += 1;
        }

        if position == start {
            return None;
        }

        let mut is_field = false;
        let value: String = self.chars[start..position].iter().collect();
        self.position = position;

        if position + 1 <= self.chars.len() {
            if let Some(lookahead) = self.chars.get(position) {
                is_field = match lookahead {
                    ':' => true,
                    '!' => self.is_operator_prefix(*lookahead, self.chars.get(position + 1).copied()),
                    '=' => true,
                    '>' => true,
                    '<' => true,
                    '~' => true,
                    '^' => true,
                    '$' => true,
                    _ => {
                        self.starts_with_keyword_from("in:", position)
                            || self.starts_with_keyword_from("has:", position)
                    }
                };
            }
        }

        if is_field {
            Some(Token::Field(value))
        } else {
            Some(Token::Value(value))
        }
    }

    fn is_operator_prefix(&self, first: char, second: Option<char>) -> bool {
        matches!(
            (first, second),
            ('!', Some('=')) | ('!', Some('~')) | ('>', Some('=')) | ('<', Some('=')) | ('~', Some('=')) | ('^', Some('=')) | ('$',
            Some('='))
        ) || first == '='
            || first == '>'
            || first == '<'
            || first == '~'
            || first == '^'
            || first == '$'
    }

    fn starts_with_keyword_from(&self, keyword: &str, position: usize) -> bool {
        let mut index = 0;
        for expected in keyword.chars() {
            if self.chars.get(position + index).copied() != Some(expected) {
                return false;
            }
            index += 1;
        }

        true
    }
}

/// Parser for filter queries
pub struct FilterParser {
    tokens: Vec<Token>,
    position: usize,
}

impl FilterParser {
    pub fn new(input: &str) -> Result<Self, ParseError> {
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize()?;

        Ok(Self {
            tokens,
            position: 0,
        })
    }

    /// Parse the filter query and return a `FilterBuilder`
    pub fn parse(&mut self) -> Result<FilterBuilder, ParseError> {
        if self.tokens.is_empty() {
            return Err(ParseError::UnexpectedEndOfInput);
        }

        let expression = self.parse_or_expression()?;

        if self.position < self.tokens.len() {
            return Err(ParseError::UnexpectedToken {
                token: format!("{:?}", self.tokens[self.position]),
                position: self.position,
            });
        }

        Ok(FilterBuilder::from_expression(expression))
    }

    fn parse_or_expression(&mut self) -> Result<FilterExpression, ParseError> {
        let mut expr = self.parse_and_expression()?;

        while self.consume_token(&Token::Or) {
            let right = self.parse_and_expression()?;
            expr = self.combine_expressions(LogicalOperator::Or, expr, right);
        }

        Ok(expr)
    }

    fn parse_and_expression(&mut self) -> Result<FilterExpression, ParseError> {
        let mut expr = self.parse_unary_expression()?;

        while self.consume_token(&Token::And) {
            let right = self.parse_unary_expression()?;
            expr = self.combine_expressions(LogicalOperator::And, expr, right);
        }

        Ok(expr)
    }

    fn parse_unary_expression(&mut self) -> Result<FilterExpression, ParseError> {
        let negated = self.consume_token(&Token::Not);
        let mut expr = if self.consume_token(&Token::LeftParen) {
            let inner = self.parse_or_expression()?;

            if !self.consume_token(&Token::RightParen) {
                return Err(ParseError::MissingClosingParen);
            }

            inner
        } else {
            self.parse_condition()?
        };

        if negated {
            expr = self.negate_expression(expr);
        }

        Ok(expr)
    }

    fn parse_condition(&mut self) -> Result<FilterExpression, ParseError> {
        let (field, implicit_operator) = self.parse_field()?;

        if self.consume_token(&Token::Colon) {
            // optional field separator
        }

        let operator = if let Some(Token::Operator(op_text)) = self.tokens.get(self.position) {
            self.position += 1;
            self.parse_operator(op_text)?
        } else if let Some(op) = implicit_operator {
            op
        } else {
            FilterOperator::Equals
        };

        let value = match operator {
            FilterOperator::IsNull | FilterOperator::IsNotNull => FilterValue::Null,
            _ => {
                match self.tokens.get(self.position) {
                    Some(Token::Value(value)) if self.is_null_value(value) => {
                        self.position += 1;
                        match operator {
                            FilterOperator::Equals => FilterValue::Null,
                            FilterOperator::NotEquals => FilterValue::Null,
                            _ => {
                                return Err(ParseError::InvalidOperatorValueCombination);
                            }
                        }
                    }
                    _ => self.parse_value(&field, &operator)?,
                }
            }
        };

        let operator = if let FilterValue::Null = value {
            match operator {
                FilterOperator::Equals => FilterOperator::IsNull,
                FilterOperator::NotEquals => FilterOperator::IsNotNull,
                op => op,
            }
        } else {
            operator
        };

        Ok(FilterExpression::Condition(FilterCondition {
            field,
            operator,
            value,
        }))
    }

    fn parse_field(&mut self) -> Result<(FilterField, Option<FilterOperator>), ParseError> {
        let token = self
            .tokens
            .get(self.position)
            .ok_or(ParseError::ExpectedField)?;

        let field_str = match token {
            Token::Field(f) => f.clone(),
            _ => return Err(ParseError::ExpectedField),
        };

        self.position += 1;

        let implicit_operator = self.implicit_operator_for_field(&field_str);

        Ok((self.parse_field_name(&field_str)?, implicit_operator))
    }

    fn implicit_operator_for_field(&self, field_str: &str) -> Option<FilterOperator> {
        match field_str.to_lowercase().as_str() {
            "has-label" => Some(FilterOperator::HasAny),
            "no-label" => Some(FilterOperator::IsNull),
            "has-assignee" => Some(FilterOperator::IsNotNull),
            "no-assignee" => Some(FilterOperator::IsNull),
            _ => None,
        }
    }

    fn parse_field_name(&self, field_str: &str) -> Result<FilterField, ParseError> {
        Ok(match field_str.to_lowercase().as_str() {
            "title" => FilterField::Title,
            "description" | "desc" => FilterField::Description,
            "status" | "state" => FilterField::Status,
            "priority" | "p" => FilterField::Priority,
            "assignee" | "assigned" | "has-assignee" | "no-assignee" => FilterField::Assignee,
            "label" | "labels" | "tag" | "tags" | "has-label" | "no-label" => FilterField::Label,
            "project" => FilterField::Project,
            "team" => FilterField::Team,
            "created" | "createdat" | "created_at" => FilterField::CreatedAt,
            "updated" | "updatedat" | "updated_at" => FilterField::UpdatedAt,
            "due" | "duedate" | "due_date" => FilterField::DueDate,
            "id" | "identifier" => FilterField::Identifier,
            _ => FilterField::Custom(field_str.to_string()),
        })
    }

    fn parse_operator(&self, op_str: &str) -> Result<FilterOperator, ParseError> {
        Ok(match op_str {
            "=" | ":" | "is" => FilterOperator::Equals,
            "!=" | "not" | "isnt" | "!" => FilterOperator::NotEquals,
            ">" | "gt" => FilterOperator::GreaterThan,
            ">=" | "gte" => FilterOperator::GreaterThanOrEquals,
            "<" | "lt" => FilterOperator::LessThan,
            "<=" | "lte" => FilterOperator::LessThanOrEquals,
            "~" | "contains" => FilterOperator::Contains,
            "!~" | "~=" => FilterOperator::NotContains,
            "^" | "^=" | "startswith" => FilterOperator::StartsWith,
            "$" | "$=" | "endswith" => FilterOperator::EndsWith,
            "in" => FilterOperator::In,
            "!in" | "notin" => FilterOperator::NotIn,
            "has" => FilterOperator::HasAny,
            "null" | "empty" => FilterOperator::IsNull,
            "!null" | "!empty" => FilterOperator::IsNotNull,
            _ => return Err(ParseError::UnknownOperator(op_str.to_string())),
        })
    }

    fn parse_value(&mut self, field: &FilterField, operator: &FilterOperator) -> Result<FilterValue, ParseError> {
        let value_str = match self.tokens.get(self.position) {
            Some(Token::Value(v)) => {
                self.position += 1;
                v.clone()
            }
            _ => return Err(ParseError::ExpectedValue),
        };

        let value_str = value_str.trim().to_string();

        match field {
            FilterField::Priority => {
                if let Ok(n) = value_str.parse::<u8>() {
                    Ok(FilterValue::Number(n as f64))
                } else {
                    match value_str.to_lowercase().as_str() {
                        "none" | "no" => Ok(FilterValue::Number(0.0)),
                        "low" => Ok(FilterValue::Number(1.0)),
                        "medium" | "med" => Ok(FilterValue::Number(2.0)),
                        "high" => Ok(FilterValue::Number(3.0)),
                        "urgent" => Ok(FilterValue::Number(4.0)),
                        _ => Err(ParseError::InvalidPriorityValue(value_str)),
                    }
                }
            }
            FilterField::CreatedAt | FilterField::UpdatedAt | FilterField::DueDate => {
                if let Some(date) = parse_relative_date(&value_str) {
                    Ok(FilterValue::Date(date))
                } else {
                    Ok(FilterValue::Date(value_str))
                }
            }
        _ => {
                if matches!(
                    operator,
                    FilterOperator::In | FilterOperator::NotIn | FilterOperator::HasAny | FilterOperator::HasNone
                ) {
                    let mut values: Vec<String> = value_str
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|v| v.to_string())
                        .collect();

                    while self.position < self.tokens.len() && self.tokens[self.position] == Token::Comma {
                        self.position += 1;
                        match self.tokens.get(self.position) {
                            Some(Token::Value(v)) => {
                                self.position += 1;
                                values.push(v.to_string());
                            }
                            _ => return Err(ParseError::ExpectedValue),
                        }
                    }

                    Ok(FilterValue::StringList(values))
                } else {
                    Ok(FilterValue::String(value_str))
                }
            }
        }
    }

    fn consume_token(&mut self, token: &Token) -> bool {
        if self.tokens.get(self.position) == Some(token) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn combine_expressions(
        &self,
        operator: LogicalOperator,
        left: FilterExpression,
        right: FilterExpression,
    ) -> FilterExpression {
        let mut conditions = Vec::new();

        match left {
            FilterExpression::Group(group) if group.operator == operator => {
                conditions.extend(group.conditions.into_iter());
            }
            other => conditions.push(other),
        }

        match right {
            FilterExpression::Group(group) if group.operator == operator => {
                conditions.extend(group.conditions.into_iter());
            }
            other => conditions.push(other),
        }

        FilterExpression::Group(Box::new(FilterGroup {
            operator,
            conditions,
        }))
    }

    fn negate_expression(&self, expr: FilterExpression) -> FilterExpression {
        match expr {
            FilterExpression::Condition(condition) => {
                let operator = self.negate_operator(condition.operator);
                FilterExpression::Condition(FilterCondition {
                    field: condition.field,
                    operator,
                    value: condition.value,
                })
            }
            FilterExpression::Group(group) => {
                FilterExpression::Group(Box::new(FilterGroup {
                    operator: LogicalOperator::Not,
                    conditions: vec![FilterExpression::Group(group)],
                }))
            }
        }
    }

    fn negate_operator(&self, operator: FilterOperator) -> FilterOperator {
        match operator {
            FilterOperator::Equals => FilterOperator::NotEquals,
            FilterOperator::NotEquals => FilterOperator::Equals,
            FilterOperator::Contains => FilterOperator::NotContains,
            FilterOperator::NotContains => FilterOperator::Contains,
            FilterOperator::IsNull => FilterOperator::IsNotNull,
            FilterOperator::IsNotNull => FilterOperator::IsNull,
            FilterOperator::In => FilterOperator::NotIn,
            FilterOperator::NotIn => FilterOperator::In,
            _ => operator,
        }
    }

    fn is_null_value(&self, value: &str) -> bool {
        matches!(value.to_lowercase().as_str(), "null" | "empty")
    }
}

/// Parse a relative date string (e.g., "7d", "2w", "1m")
fn parse_relative_date(input: &str) -> Option<String> {
    let re = Regex::new(r"^(\d+)([hdwmHDWM])$").unwrap();
    if let Some(captures) = re.captures(input) {
        let amount = captures[1].parse::<i64>().ok()?;
        let unit = captures[2].to_lowercase();

        let duration = match unit.as_str() {
            "h" => Duration::hours(amount),
            "d" => Duration::days(amount),
            "w" => Duration::weeks(amount),
            "m" => Duration::days(amount * 30),
            _ => return None,
        };

        let date = Utc::now() - duration;
        return Some(date.to_rfc3339());
    }

    None
}

/// Parse errors
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected character at position {position}: '{char}'")]
    UnexpectedCharacter { position: usize, char: char },

    #[error("Unexpected token at position {position}: {token}")]
    UnexpectedToken { token: String, position: usize },

    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,

    #[error("Unterminated quoted string at position {position}")]
    UnterminatedString { position: usize },

    #[error("Expected field name")]
    ExpectedField,

    #[error("Expected value")]
    ExpectedValue,

    #[error("Unknown operator: {0}")]
    UnknownOperator(String),

    #[error("Invalid priority value: {0}")]
    InvalidPriorityValue(String),

    #[error("Missing closing parenthesis")]
    MissingClosingParen,

    #[error("Invalid operator/value combination")]
    InvalidOperatorValueCombination,
}

/// Parse a filter query string into a FilterBuilder
pub fn parse_filter(query: &str) -> Result<FilterBuilder, ParseError> {
    let mut parser = FilterParser::new(query)?;
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_filter() {
        let builder = parse_filter("status:completed").unwrap();
        assert!(matches!(builder.to_graphql(), Ok(_)));
    }

    #[test]
    fn test_compound_filter() {
        let builder = parse_filter("status!=completed AND priority>2").unwrap();
        assert!(matches!(builder.to_graphql(), Ok(_)));
    }

    #[test]
    fn test_relative_dates() {
        let builder = parse_filter("created>7d AND updated<2w").unwrap();
        assert!(matches!(builder.to_graphql(), Ok(_)));
    }

    #[test]
    fn test_quoted_values() {
        let builder = parse_filter(r#"title~"bug fix" AND assignee="john@example.com""#).unwrap();
        assert!(matches!(builder.to_graphql(), Ok(_)));
    }

    #[test]
    fn test_list_values() {
        let builder = parse_filter("status in:backlog,unstarted,started").unwrap();
        assert!(matches!(builder.to_graphql(), Ok(_)));
    }

    #[test]
    fn test_legacy_has_label_filter() {
        let builder = parse_filter("has-label:urgent").unwrap();
        let graphql = builder.to_graphql().unwrap();

        assert!(graphql.get("labels").is_some());
        let labels = graphql.get("labels").unwrap();
        assert!(labels.get("some").is_some() || labels.get("every").is_some());
    }

    #[test]
    fn test_legacy_no_assignee_filter() {
        let builder = parse_filter("no-assignee").unwrap();
        let graphql = builder.to_graphql().unwrap();

        let assignee = graphql.get("assignee").unwrap();
        assert_eq!(assignee["null"], true);
    }

    #[test]
    fn test_legacy_no_label_filter() {
        let builder = parse_filter("no-label").unwrap();
        let graphql = builder.to_graphql().unwrap();

        let labels = graphql.get("labels").unwrap();
        assert!(labels.get("every").is_some());
    }

    #[test]
    fn test_negation() {
        let builder = parse_filter("NOT status:completed").unwrap();
        assert!(matches!(builder.to_graphql(), Ok(_)));
    }

    #[test]
    fn test_parentheses() {
        let builder = parse_filter("(priority>2 OR label:urgent) AND NOT status:completed").unwrap();
        assert!(matches!(builder.to_graphql(), Ok(_)));
    }

    #[test]
    fn test_and_or_precedence() {
        let builder = parse_filter("status:done OR priority>2 AND assignee:john").unwrap();
        let graphql = builder.to_graphql().unwrap();
        let graphql_string = graphql.to_string();
        assert!(graphql_string.contains("\"or\""));
    }

    #[test]
    fn test_utf8_tokenization_in_values() {
        let mut tokenizer = Tokenizer::new("title~\"bug 🐛\" assignee:jose@example.com");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(matches!(&tokens[0], Token::Field(field) if field == "title"));
        assert!(matches!(&tokens[1], Token::Operator(op) if op == "~"));
        assert!(matches!(&tokens[2], Token::Value(value) if value == "bug 🐛"));
    }

    #[test]
    fn test_tokenizer_in_operator() {
        let mut tokenizer = Tokenizer::new("status in:backlog,unstarted");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], Token::Field(f) if f == "status"));
        assert!(matches!(&tokens[1], Token::Operator(op) if op == "in"));
        assert!(matches!(&tokens[2], Token::Value(v) if v == "backlog,unstarted"));
    }
}
