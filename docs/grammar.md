# Lox Grammar Rules

```shell
program        → declaration* EOF ;

declaration    → classDecl
               | funDecl
               | varDecl
               | statement ;

classDecl      → "class" IDENTIFIER ( "<" IDENTIFIER )?
                 "{" function* "}" ;

funDecl        → "fun" (function | lambda) ;
function       → IDENTIFIER "(" parameters? ")" block ;
lambda         → "(" parameters? ")" block ( "(" arguments? ")" )* ";" ;
parameters     → IDENTIFIER ( "," IDENTIFIER )* ;

varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;

statement      → exprStmt
               | ifStmt
               | printStmt 
               | returnStmt
               | whileStmt
               | forStmt
               | block ;

returnStmt     → "return" expression? ";" ;

ifStmt         → "if" "(" expression ")" statement
               ( "else" statement )? ;
whileStmt      → "while" "(" expression ")" statement ;

forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
                 expression? ";"
                 expression? ")" statement ;

block          → "{" declaration* "}" ;

exprStmt       → expression ";" ;
printStmt      → "print" expression ";" ;

expression     → sequence ;
sequence       → assignment ( ( "," ) expression )* ; // Chp 6+
assignment     → ( call "." )? IDENTIFIER "=" assignment | logic_or ;

logic_or       → logic_and ( "or" logic_and )* ;
logic_and      → equality ( "and" equality )* ;

equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary
               | call ;

call           → lambdaExpr ( "(" arguments? ")" | "." IDENTIFIER )* ;
lambdaExpr     → "fun" "(" parameters? ")" block | primary ; // Chp 10+
arguments      → expression ( "," expression )* ;

primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")" 
               | IDENTIFIER 
               | "super" "." IDENTIFIER ;

```