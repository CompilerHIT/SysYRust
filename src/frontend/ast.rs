#[derive(Debug)]
pub struct CompUnit {
    pub global_items: Vec<GlobalItems>,
}

#[derive(Debug)]
pub enum GlobalItems {
    Decl(Decl),
    FuncDef(FuncDef),
}

#[derive(Debug)]
pub enum Decl {
    ConstDecl(ConstDecl),
    VarDecl(VarDecl),
}

#[derive(Debug)]
pub struct ConstDecl {
    pub btype: BType,
    pub const_def_vec: Vec<ConstDef>,
}

#[derive(Debug)]
pub enum BType {
    Int,
    Float,
}

#[derive(Debug)]
pub struct ConstDef {
    pub ident: Ident,
    pub const_exp_vec: Vec<ConstExp>,
    pub const_init_val: ConstInitVal,
}

#[derive(Debug)]
pub enum ConstInitVal {
    ConstExp(ConstExp),
    ConstInitValVec(Vec<ConstInitVal>),
}

#[derive(Debug)]
pub struct VarDecl {
    pub btype: BType,
    pub var_def_vec: Vec<VarDef>,
}

#[derive(Debug)]
pub enum VarDef {
    NonArray(Ident),
    Array((Ident, Vec<ConstExp>)),
    NonArrayInit((Ident, InitVal)),
    ArrayInit((Ident, Vec<ConstExp>, InitVal)),
}

#[derive(Debug)]
pub enum InitVal {
    Exp(Exp),
    InitValVec(Vec<InitVal>),
}

#[derive(Debug)]
pub enum FuncDef {
    NonParameterFuncDef((FuncType, Ident, Block)),
    ParameterFuncDef((FuncType, Ident, FuncFParams, Block)),
}

#[derive(Debug)]
pub enum FuncType {
    Void,
    Int,
    Float,
}

#[derive(Debug)]
pub struct FuncFParams {
    pub func_fparams_vec: Vec<FuncFParam>,
}

#[derive(Debug)]
pub enum FuncFParam {
    NonArray((BType, Ident)),
    Array((BType, Ident, Vec<Exp>)),
}

#[derive(Debug)]
pub struct Block {
    pub block_vec: Vec<BlockItem>,
}

#[derive(Debug)]
pub enum BlockItem {
    Decl(Decl),
    Stmt(Stmt),
}

#[derive(Debug)]
pub enum Stmt {
    Assign(Assign),
    ExpStmt(ExpStmt),
    Block(Block),
    If(Box<If>),
    While(Box<While>),
    Break(Break),
    Continue(Continue),
    Return(Return),
}

#[derive(Debug)]
pub struct Assign {
    pub lval: LVal,
    pub exp: Exp,
}

#[derive(Debug)]
pub struct ExpStmt {
    pub exp: Option<Exp>,
}

#[derive(Debug)]
pub struct If {
    pub cond: Cond,
    pub then: Stmt,
    pub else_then: Option<Stmt>,
}

#[derive(Debug)]
pub struct While {
    pub cond: Cond,
    pub body: Stmt,
}

#[derive(Debug)]
pub struct Break;

#[derive(Debug)]
pub struct Continue;

#[derive(Debug)]
pub struct Return {
    pub exp: Option<Exp>,
}

#[derive(Debug)]
pub struct Exp {
    pub add_exp: Box<AddExp>,
}

#[derive(Debug)]
pub struct Cond {
    pub l_or_exp: LOrExp,
}

#[derive(Debug)]
pub struct LVal {
    pub id: Ident,
    pub exp_vec: Vec<Exp>,
}

#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    LVal(LVal),
    Number(Number),
}

#[derive(Debug)]
pub enum Number {
    IntConst(IntConst),
    FloatConst(FloatConst),
}

#[derive(Debug)]
pub struct OptionFuncRParams {
    pub func_fparams: Option<FuncRParams>,
}

#[derive(Debug)]
pub enum UnaryExp {
    PrimaryExp(Box<PrimaryExp>),
    FuncCall((Ident, OptionFuncRParams)),
    OpUnary((UnaryOp, Box<UnaryExp>)),
}

#[derive(Debug)]
pub enum UnaryOp {
    Add,
    Minus,
    Exclamation,
}

#[derive(Debug)]
pub struct FuncRParams {
    pub exp_vec: Vec<Exp>,
}

#[derive(Debug)]
pub enum MulExp {
    UnaryExp(Box<UnaryExp>),
    MulExp((Box<MulExp>, UnaryExp)),
    DivExp((Box<MulExp>, UnaryExp)),
    ModExp((Box<MulExp>, UnaryExp)),
}

#[derive(Debug)]
pub enum AddOp {
    Add,
    Minus,
}

#[derive(Debug)]
pub enum AddExp {
    MulExp(Box<MulExp>),
    OpExp((Box<AddExp>, AddOp, MulExp)),
}

#[derive(Debug)]
pub enum RelOp {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

#[derive(Debug)]
pub enum RelExp {
    AddExp(AddExp),
    OpExp((Box<RelExp>, RelOp, AddExp)),
}

#[derive(Debug)]
pub enum EqExp {
    RelExp(RelExp),
    EqualExp((Box<EqExp>, RelExp)),
    NotEqualExp((Box<EqExp>, RelExp)),
}

#[derive(Debug)]
pub enum LAndExp {
    EqExp(EqExp),
    AndExp((Box<LAndExp>, EqExp)),
}

#[derive(Debug)]
pub struct ConstExp {
    pub add_exp: AddExp,
}

#[derive(Debug)]
pub enum LOrExp {
    LAndExp(LAndExp),
    OrExp((Box<LOrExp>, LAndExp)),
}

pub type Ident = String;

pub type IntConst = i32;
pub type FloatConst = f32;
