use crate::Layers;
use crate::{RESET, UF2};

pub static LAYERS: Layers = keyberon::layout::layout! {
    {
        [      Up     Z  X  ],
        [ Left Down   Right ],
        [ (1)  LShift Space ] ,
    }
    {
        [    {RESET}  t t   ],
        [ n  n        {UF2} ],
        [ t  t        t     ],
    }
};