use super::namespace::*;
use crate::ast::ast_def::*;

pub trait ConstEvaluator {
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32>;
}

impl ConstEvaluator for ConstExpr {
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32>{
        return self.expr.const_eval(namesp);
    }
}

impl ConstEvaluator for Expr {
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32>{
        match self {
            Self::LOr(expr) => expr.const_eval(namesp),
        }
    }
}

impl ConstEvaluator for LOrExpr {
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32>{
        match self {
            Self::LAndExpr(land_expr) => land_expr.const_eval(namesp),

            Self::LOrExpr(lexpr, rexpr) => {
                let lval = lexpr.const_eval(namesp);
                let rval = rexpr.const_eval(namesp);
                match (lval, rval) {
                    (Some(lval), Some(rval)) => Some((lval!=0 || rval!=0) as i32),
                    _ => None,
                }
            }
        }
    }
}

impl ConstEvaluator for LAndExpr{
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32> {
        match self {
            Self::EqExpr(eqexpr) => eqexpr.const_eval(namesp),

            Self::LAndExpr(lexpr, rexpr) => {
                let lval = lexpr.const_eval(namesp);
                let rval = rexpr.const_eval(namesp);
                match (lval, rval) {
                    (Some(lval), Some(rval)) => Some((lval!=0 && rval!=0) as i32),
                    _ => None,
                }
            }
        }
    }
}

impl ConstEvaluator for EqExpr{
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32> {
        match self {
            Self::RelExpr(expr) => expr.const_eval(namesp),

            Self::EqExpr(lexpr, op, rexpr) => {
                let lval = lexpr.const_eval(namesp);
                let rval = rexpr.const_eval(namesp);
                match (lval, rval) {
                    (Some(lval), Some(rval)) => {
                        match op {
                            EqOp::Eq => Some((lval == rval) as i32),
                            EqOp::Ne => Some((lval != rval) as i32),
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}

impl ConstEvaluator for RelExpr {
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32> {
        match self {
            Self::AddExpr(expr) => expr.const_eval(namesp),

            Self::RelExpr(lexpr, op, rexpr) => {
                let lval = lexpr.const_eval(namesp);
                let rval = rexpr.const_eval(namesp);
                match (lval, rval) {
                    (Some(lval), Some(rval)) => {
                        match op {
                            RelOp::Lt => Some((lval < rval) as i32),
                            RelOp::Gt => Some((lval > rval) as i32),
                            RelOp::Le => Some((lval <= rval) as i32),
                            RelOp::Ge => Some((lval >= rval) as i32),
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}

impl ConstEvaluator for AddExpr {
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32> {
        match self {
            Self::MulExpr(expr) => expr.const_eval(namesp),

            Self::AddAndMul(lexpr, op, rexpr) => {
                let lval = lexpr.const_eval(namesp);
                let rval = rexpr.const_eval(namesp);
                match (lval, rval) {
                    (Some(lval), Some(rval)) => {
                        match op {
                            AddOp::Add => Some(lval + rval),
                            AddOp::Minus => Some(lval - rval),
                        }
                    }
                    _ => None,
                }
            }
        }
}
}

impl ConstEvaluator for MulExpr {
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32> {
        match self {
            Self::UnaryExpr(expr) => expr.const_eval(namesp),

            Self::MulAndUnary(lexpr, op, rexpr) => {
                let lval = lexpr.const_eval(namesp);
                let rval = rexpr.const_eval(namesp);
                match (lval, rval) {
                    (Some(lval), Some(rval)) => {
                        match op {
                            MulOp::Mul => Some(lval * rval),
                            MulOp::Div => (rval!=0).then_some(lval / rval),
                            MulOp::Mod => (rval!=0).then_some(lval % rval),
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}

impl ConstEvaluator for UnaryExpr {
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32> {
        match self {
            Self::PrimExpr(expr) => expr.const_eval(namesp),
            Self::FuncCall(_) => None,

            Self::UnaryExpr(op, expr) => expr.const_eval(namesp).map(|val| {
                match op {
                    UnaryOp::Pos => val,
                    UnaryOp::Neg => -val,
                    UnaryOp::Not => (val==0) as i32,
                }
            }),
        }
    }
}

impl ConstEvaluator for PrimExpr {
    fn const_eval(&self, namesp: &mut Namesp) -> Option<i32> {
        match self {
            Self::Expr(expr) => expr.const_eval(namesp),
            Self::LVal(lval) => lval.const_eval(namesp),
            Self::Number(num) => Some(*num),
        }
    }
}

impl ConstEvaluator for LVal {
    fn const_eval(&self ,namesp: &mut Namesp) -> Option<i32>{
        let val = namesp.get_value(&self.id);
        if val.is_err() {
            return None;
        }

        let val = val.unwrap();
        if self.inds.len() != 0 {
            panic!("Array unit cant be const");
        }
        match val {
            NamespValue::ConstInt(val) => Some(*val),
            _ => None,
        }

    }
}