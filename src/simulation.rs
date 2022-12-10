#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationState {
    Play,
    Pause,
    Finish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FightResults {
    Won,
    Tied,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Rock,
    Paper,
    Scissors,
}

impl Shape {
    pub fn fight(self, other: Self) -> FightResults {
        if self == other {
            FightResults::Tied
        } else if ((other as isize + 1) % 3) == self as isize {
            FightResults::Won
        } else {
            FightResults::Lost
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fight() {
        use super::{FightResults::*, Shape::*};

        assert_eq!(Shape::fight(Paper, Rock), Won);
        assert_eq!(Shape::fight(Rock, Rock), Tied);
        assert_eq!(Shape::fight(Scissors, Rock), Lost);
    }
}
