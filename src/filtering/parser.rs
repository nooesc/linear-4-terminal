use chrono::{Duration, Utc};
use regex::Regex;

use super::builder::{FilterBuilder, FilterField, FilterOperator, FilterValue};

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

/// Tokenizer for filter queries
struct Tokenizer {
    input: String,
    position: usize,
    last_operator: Option<String>,
}

impl Tokenizer {
    fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            position: 0,
            last_operator: None,
        }
    }
    
    fn tokenize(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        
        while self.position < self.input.len() {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                break;
            }
            
            // Check for operators and keywords
            if let Some(token) = self.try_parse_operator() {
                tokens.push(token);
            } else if let Some(token) = self.try_parse_keyword() {
                tokens.push(token);
            } else if let Some(token) = self.try_parse_special() {
                tokens.push(token);
            } else if let Some(token) = self.try_parse_value() {
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
        while self.position < self.input.len() && self.current_char().unwrap_or(' ').is_whitespace() {
            self.position += 1;
        }
    }
    
    fn current_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }
    
    fn peek_string(&self, len: usize) -> Option<String> {
        if self.position + len > self.input.len() {
            return None;
        }
        Some(self.input[self.position..self.position + len].to_string())
    }
    
    fn try_parse_operator(&mut self) -> Option<Token> {
        // Check for two-character operators first
        if let Some(s) = self.peek_string(2) {
            let op = match s.as_str() {
                "!=" => Some("!="),
                ">=" => Some(">="),
                "<=" => Some("<="),
                "~=" => Some("~="),
                "^=" => Some("^="),
                "$=" => Some("$="),
                _ => None,
            };
            
            if let Some(op) = op {
                self.position += 2;
                self.last_operator = Some(op.to_string());
                return Some(Token::Operator(op.to_string()));
            }
        }
        
        // Single character operators
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
            _ => None,
        }
    }
    
    fn try_parse_keyword(&mut self) -> Option<Token> {
        let remaining = &self.input[self.position..];
        
        // Try to match keywords
        if remaining.to_lowercase().starts_with("and") && self.is_word_boundary(self.position + 3) {
            self.position += 3;
            return Some(Token::And);
        }
        
        if remaining.to_lowercase().starts_with("or") && self.is_word_boundary(self.position + 2) {
            self.position += 2;
            return Some(Token::Or);
        }
        
        if remaining.to_lowercase().starts_with("not") && self.is_word_boundary(self.position + 3) {
            self.position += 3;
            return Some(Token::Not);
        }
        
        // Try to match special operators
        if remaining.starts_with("in:") {
            self.position += 3;
            self.last_operator = Some("in".to_string());
            return Some(Token::Operator("in".to_string()));
        }
        
        if remaining.starts_with("has:") {
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
    
    fn try_parse_value(&mut self) -> Option<Token> {
        let start = self.position;
        
        // Check if value is quoted
        if self.current_char() == Some('"') {
            self.position += 1;
            while self.position < self.input.len() && self.current_char() != Some('"') {
                if self.current_char() == Some('\\') {
                    self.position += 2; // Skip escaped character
                } else {
                    self.position += 1;
                }
            }
            
            if self.current_char() == Some('"') {
                self.position += 1;
                let value = self.input[start + 1..self.position - 1].to_string();
                return Some(Token::Value(value));
            }
        }
        
        // Parse unquoted value
        // If last operator was "in", include commas in the value
        let include_commas = self.last_operator.as_deref() == Some("in");
        
        while self.position < self.input.len() {
            match self.current_char() {
                Some(c) if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '@' => {
                    self.position += 1;
                }
                Some(',') if include_commas => {
                    self.position += 1;
                }
                _ => break,
            }
        }
        
        if self.position > start {
            let value = self.input[start..self.position].to_string();
            
            // Determine if it's a field or value based on what follows
            let saved_pos = self.position;
            self.skip_whitespace();
            
            // Check if followed by an operator (including special ones like "in:")
            let is_field = self.current_char() == Some(':') 
                || self.peek_string(2) == Some("!=".to_string()) 
                || self.peek_string(2) == Some(">=".to_string()) 
                || self.peek_string(2) == Some("<=".to_string())
                || self.current_char() == Some('>')
                || self.current_char() == Some('<')
                || self.current_char() == Some('=')
                || self.current_char() == Some('~')
                || self.peek_string(3) == Some("in:".to_string())
                || self.peek_string(4) == Some("has:".to_string());
                
            self.position = saved_pos; // Restore position
            
            if is_field {
                Some(Token::Field(value))
            } else {
                Some(Token::Value(value))
            }
        } else {
            None
        }
    }
    
    fn is_word_boundary(&self, pos: usize) -> bool {
        if pos >= self.input.len() {
            return true;
        }
        
        match self.input.chars().nth(pos) {
            Some(c) if c.is_alphanumeric() => false,
            _ => true,
        }
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
    
    /// Parse the filter query and return a FilterBuilder
    pub fn parse(&mut self) -> Result<FilterBuilder, ParseError> {
        let mut builder = FilterBuilder::new();
        
        // Parse the expression
        self.parse_expression(&mut builder)?;
        
        // Ensure we consumed all tokens
        if self.position < self.tokens.len() {
            return Err(ParseError::UnexpectedToken {
                token: format!("{:?}", self.tokens[self.position]),
                position: self.position,
            });
        }
        
        Ok(builder)
    }
    
    fn parse_expression(&mut self, builder: &mut FilterBuilder) -> Result<(), ParseError> {
        // Parse first condition
        self.parse_condition(builder)?;
        
        // Parse additional conditions with operators
        while self.position < self.tokens.len() {
            match &self.tokens[self.position] {
                Token::And => {
                    self.position += 1;
                    builder.and();
                    self.parse_condition(builder)?;
                }
                Token::Or => {
                    self.position += 1;
                    builder.or();
                    self.parse_condition(builder)?;
                }
                _ => break,
            }
        }
        
        Ok(())
    }
    
    fn parse_condition(&mut self, builder: &mut FilterBuilder) -> Result<(), ParseError> {
        // Handle NOT operator
        let negated = if self.position < self.tokens.len() && self.tokens[self.position] == Token::Not {
            self.position += 1;
            true
        } else {
            false
        };
        
        // Handle parentheses for grouping
        if self.position < self.tokens.len() && self.tokens[self.position] == Token::LeftParen {
            self.position += 1;
            
            if negated {
                builder.not_group();
            }
            
            self.parse_expression(builder)?;
            
            if self.position >= self.tokens.len() || self.tokens[self.position] != Token::RightParen {
                return Err(ParseError::MissingClosingParen);
            }
            self.position += 1;
            
            if negated {
                builder.end_group();
            }
            
            return Ok(());
        }
        
        // Parse field
        let field = match &self.tokens.get(self.position) {
            Some(Token::Field(f)) => {
                self.position += 1;
                self.parse_field(f)?
            }
            _ => return Err(ParseError::ExpectedField),
        };
        
        // Parse operator (optional colon)
        if self.position < self.tokens.len() && self.tokens[self.position] == Token::Colon {
            self.position += 1;
        }
        
        // Check if the next token is a special value that acts as an operator
        let (operator, value) = if let Some(Token::Value(val)) = self.tokens.get(self.position) {
            match val.as_str() {
                "null" | "empty" => {
                    self.position += 1;
                    (FilterOperator::IsNull, FilterValue::Null)
                }
                _ => {
                    // Parse operator
                    let op = match self.tokens.get(self.position) {
                        Some(Token::Operator(op)) => {
                            self.position += 1;
                            self.parse_operator(op)?
                        }
                        _ => FilterOperator::Equals, // Default operator
                    };
                    
                    // Parse value
                    let val = self.parse_value(&field, &op)?;
                    (op, val)
                }
            }
        } else {
            // Parse operator
            let op = match self.tokens.get(self.position) {
                Some(Token::Operator(op)) => {
                    self.position += 1;
                    self.parse_operator(op)?
                }
                _ => FilterOperator::Equals, // Default operator
            };
            
            // Parse value
            let val = self.parse_value(&field, &op)?;
            (op, val)
        };
        
        // Apply the condition using the builder
        self.apply_condition(builder, field, operator, value, negated)?;
        
        Ok(())
    }
    
    fn parse_field(&self, field_str: &str) -> Result<FilterField, ParseError> {
        Ok(match field_str.to_lowercase().as_str() {
            "title" => FilterField::Title,
            "description" | "desc" => FilterField::Description,
            "status" | "state" => FilterField::Status,
            "priority" | "p" => FilterField::Priority,
            "assignee" | "assigned" => FilterField::Assignee,
            "label" | "labels" | "tag" | "tags" => FilterField::Label,
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
            "!=" | "not" | "isnt" => FilterOperator::NotEquals,
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
        // Handle special cases
        match operator {
            FilterOperator::IsNull | FilterOperator::IsNotNull => return Ok(FilterValue::Null),
            _ => {}
        }
        
        // Parse value token
        let value_str = match self.tokens.get(self.position) {
            Some(Token::Value(v)) => {
                self.position += 1;
                v.clone()
            }
            _ => return Err(ParseError::ExpectedValue),
        };
        
        // Convert based on field type
        match field {
            FilterField::Priority => {
                // Try to parse as number
                if let Ok(n) = value_str.parse::<u8>() {
                    Ok(FilterValue::Number(n as f64))
                } else {
                    // Convert priority names to numbers
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
                // Try to parse relative date
                if let Some(date) = parse_relative_date(&value_str) {
                    Ok(FilterValue::Date(date))
                } else {
                    // Assume it's an absolute date
                    Ok(FilterValue::Date(value_str))
                }
            }
            _ => {
                // Handle list values for IN operators
                if matches!(operator, FilterOperator::In | FilterOperator::NotIn) {
                    let mut values = vec![value_str];
                    
                    // Parse additional comma-separated values
                    while self.position < self.tokens.len() && self.tokens[self.position] == Token::Comma {
                        self.position += 1;
                        match self.tokens.get(self.position) {
                            Some(Token::Value(v)) => {
                                self.position += 1;
                                values.push(v.clone());
                            }
                            _ => break,
                        }
                    }
                    
                    Ok(FilterValue::StringList(values))
                } else {
                    Ok(FilterValue::String(value_str))
                }
            }
        }
    }
    
    fn apply_condition(
        &self,
        builder: &mut FilterBuilder,
        field: FilterField,
        operator: FilterOperator,
        value: FilterValue,
        negated: bool,
    ) -> Result<(), ParseError> {
        // Get field builder
        let field_builder = builder.field(field);
        
        // Apply operator with potential negation
        let effective_operator = if negated {
            match operator {
                FilterOperator::Equals => FilterOperator::NotEquals,
                FilterOperator::NotEquals => FilterOperator::Equals,
                FilterOperator::Contains => FilterOperator::NotContains,
                FilterOperator::NotContains => FilterOperator::Contains,
                FilterOperator::IsNull => FilterOperator::IsNotNull,
                FilterOperator::IsNotNull => FilterOperator::IsNull,
                FilterOperator::In => FilterOperator::NotIn,
                FilterOperator::NotIn => FilterOperator::In,
                _ => operator, // Some operators don't have direct negations
            }
        } else {
            operator
        };
        
        // Apply the condition based on operator
        match (effective_operator, value) {
            (FilterOperator::Equals, v) => { field_builder.equals(v); }
            (FilterOperator::NotEquals, v) => { field_builder.not_equals(v); }
            (FilterOperator::GreaterThan, v) => { field_builder.greater_than(v); }
            (FilterOperator::GreaterThanOrEquals, v) => { field_builder.greater_than_or_equals(v); }
            (FilterOperator::LessThan, v) => { field_builder.less_than(v); }
            (FilterOperator::LessThanOrEquals, v) => { field_builder.less_than_or_equals(v); }
            (FilterOperator::Contains, FilterValue::String(s)) => { field_builder.contains(s); }
            (FilterOperator::NotContains, FilterValue::String(s)) => { field_builder.not_contains(s); }
            (FilterOperator::StartsWith, FilterValue::String(s)) => { field_builder.starts_with(s); }
            (FilterOperator::EndsWith, FilterValue::String(s)) => { field_builder.ends_with(s); }
            (FilterOperator::In, FilterValue::StringList(list)) => { field_builder.in_list(list); }
            (FilterOperator::NotIn, FilterValue::StringList(list)) => { field_builder.not_in_list(list); }
            (FilterOperator::IsNull, _) => { field_builder.is_null(); }
            (FilterOperator::IsNotNull, _) => { field_builder.is_not_null(); }
            _ => return Err(ParseError::InvalidOperatorValueCombination),
        }
        
        Ok(())
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
            "m" => Duration::days(amount * 30), // Approximation
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
        // Test would verify the builder structure
    }
    
    #[test]
    fn test_compound_filter() {
        let builder = parse_filter("status!=completed AND priority>2").unwrap();
        // Test would verify the builder structure
    }
    
    #[test]
    fn test_relative_dates() {
        let builder = parse_filter("created>7d AND updated<2w").unwrap();
        // Test would verify the builder structure
    }
    
    #[test]
    fn test_quoted_values() {
        let builder = parse_filter(r#"title~"bug fix" AND assignee="john@example.com""#).unwrap();
        // Test would verify the builder structure
    }
    
    #[test]
    fn test_list_values() {
        let builder = parse_filter("status in:backlog,unstarted,started").unwrap();
        // Test would verify the builder structure
    }
    
    #[test]
    fn test_negation() {
        let builder = parse_filter("NOT status:completed").unwrap();
        // Test would verify the builder structure
    }
    
    #[test]
    fn test_parentheses() {
        let builder = parse_filter("(priority>2 OR urgent) AND NOT status:completed").unwrap();
        // Test would verify the builder structure
    }
    
    #[test]
    fn test_tokenizer_in_operator() {
        let mut tokenizer = Tokenizer::new("status in:backlog,unstarted");
        let tokens = tokenizer.tokenize().unwrap();
        println!("Tokens: {:?}", tokens);
        
        // We expect: [Field("status"), Operator("in"), Value("backlog,unstarted")]
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], Token::Field(f) if f == "status"));
        assert!(matches!(&tokens[1], Token::Operator(op) if op == "in"));
        assert!(matches!(&tokens[2], Token::Value(v) if v == "backlog,unstarted"));
    }
}