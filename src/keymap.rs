use crate::Layers;
use crate::{RESET, UF2, CHANGE_DEFAULT_LAYER};

pub static LAYERS: Layers = keyberon::layout::layout! {
    // layer 0: m8.run keys
    {
        [      Up     Z  X  ],
        [ Left Down   Right ],
        [ (2)  LShift Space ] ,
    }
    // layer 1: m8c keys
    {
        [      Up     A  S  ],
        [ Left Down   Right ],
        [ (2)  LShift Space ] ,
    }
    // layer 2: special layer (activated by Left+Down chord)
    {
        [    t t {UF2} ],
        [ n  n {CHANGE_DEFAULT_LAYER} ],
        [ t  t {RESET} ],
    }
};