// Implemented:
// CompUnit, Decl, Def, Func, Stmt
// Initialize Symbol
#[derive(Debug)]
pub struct CompileInit {
    pub init: Vec<DeclOrFunc>,
}

#[derive(Debug)]
pub enum DeclOrFunc {
    Decl(Decl),
    Func(FuncDef),
}

#[derive(Debug)]
pub enum Decl {
    Const(ConstDecl),
    Var(VarDecl),
}
#[derive(Debug)]
pub struct ConstDecl {
    pub defs: Vec<ConstDef>,
}
#[derive(Debug)]
pub struct ConstDef {
    pub id: String,
    pub dims: Vec<ConstExpr>,
    pub init_val: ConstInitVal,
}
#[derive(Debug)]
pub enum ConstInitVal {
    Expr(ConstExpr),
    List(Vec<ConstInitVal>),
}
#[derive(Debug)]
pub struct VarDecl {
    pub defs: Vec<VarDef>,
}
#[derive(Debug)]
pub struct VarDef {
    pub id: String,
    pub dims: Vec<ConstExpr>,
    pub init_val: Option<InitVal>,
}
#[derive(Debug)]
pub enum InitVal {
    Expr(Expr),
    List(Vec<InitVal>),
}

// Function Defination    ( int/void funcName(parType1 parName1, ...){...} )
#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub func_name: String,
    pub func_params: Vec<Param>,
    pub func_body: Block,
}
//
#[derive(Debug)]
pub enum FuncType {
    Void,
    Int,
}
// Function Parameters
#[derive(Debug)]
pub struct Param {
    pub param_id: String,
    pub param_dims: Option<Vec<ConstExpr>>,
}
// Code Block {}
#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
}
#[derive(Debug)]
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt),
}

// Statement
#[derive(Debug)]
pub enum Stmt {
    ReturnStmt(ReturnStmt),
    AssignStmt(AssignStmt),
    ExprStmt(ExprStmt),
    BlockStmt(Block),
    IfStmt(Box<IfStmt>),
    WhileStmt(Box<WhileStmt>),
    BreakStmt(BreakStmt),
    ContinueStmt(ContinueStmt),
}

// Statement Type: Return    ( return expr; )
#[derive(Debug)]
pub struct ReturnStmt {
    pub expr: Option<Expr>,
}

// Statement Type: Assign    ( lval = rval; )
#[derive(Debug)]
pub struct AssignStmt {
    pub lval: LVal,
    pub expr: Expr,
}

// Statement Type: Expr    ( expr; )
#[derive(Debug)]
pub struct ExprStmt {
    pub expr: Option<Expr>,
}

// Statement Type: If    ( if() {} else {} )
#[derive(Debug)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_stmt: Stmt, // Block or Single Stmt
    pub else_stmt: Stmt,
}

// Statement Type: While    ( while() {})
#[derive(Debug)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body_stmt: Stmt, // Block or Single Stmt
}

// Statement Type: Break    ( break; )
#[derive(Debug)]
pub struct BreakStmt;

// Statement Type: Continue    ( continue; )
#[derive(Debug)]
pub struct ContinueStmt;

// Lv.1

#[derive(Debug)]
pub enum ASTType {
    Int,
}

#[derive(Debug)]
pub enum Expr {
    LOr(LOrExpr),
}

#[derive(Debug)]
pub enum LOrExpr {
    LAndExpr(LAndExpr),
    LOrExpr(Box<LOrExpr>, LAndExpr),
}

#[derive(Debug)]
pub enum LAndExpr {
    EqExpr(EqExpr),
    LAndExpr(Box<LAndExpr>, EqExpr),
}

#[derive(Debug)]
pub enum EqExpr {
    RelExpr(RelExpr),
    EqExpr(Box<EqExpr>, EqOp, RelExpr),
}

#[derive(Debug)]
pub enum EqOp {
    Eq,
    Ne,
}

#[derive(Debug)]
pub enum RelExpr {
    AddExpr(AddExpr),
    RelExpr(Box<RelExpr>, RelOp, AddExpr),
}

#[derive(Debug)]
pub enum RelOp {
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug)]
pub enum AddExpr {
    MulExpr(MulExpr),
    AddAndMul(Box<AddExpr>, AddOp, MulExpr),
}

#[derive(Debug)]
pub enum AddOp {
    Add,
    Minus,
}

#[derive(Debug)]
pub enum MulExpr {
    UnaryExpr(UnaryExpr),
    MulAndUnary(Box<MulExpr>, MulOp, UnaryExpr),
}

#[derive(Debug)]
pub enum MulOp {
    Mul,
    Div,
    Mod,
}

#[derive(Debug)]
pub enum UnaryExpr {
    PrimExpr(PrimExpr),
    FuncCall(FuncCall),
    UnaryExpr(UnaryOp, Box<UnaryExpr>),
}

#[derive(Debug)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

#[derive(Debug)]
pub struct FuncCall {
    pub funcid: String,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub enum PrimExpr {
    Expr(Box<Expr>),
    LVal(LVal),
    Number(i32),
}

#[derive(Debug)]
pub struct LVal {
    pub id: String,
    pub inds: Vec<Expr>,
}
#[derive(Debug)]
pub struct ConstExpr {
    pub expr: Expr,
}
