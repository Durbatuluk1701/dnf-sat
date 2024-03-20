pub mod formula {
    #[derive(Hash, Eq, PartialEq, Debug, Clone)]
    pub enum Formula {
        FVar(u32),
        FNeg(Box<Formula>),
        FDisj(Box<Formula>, Box<Formula>),
        FConj(Box<Formula>, Box<Formula>),
    }
}
