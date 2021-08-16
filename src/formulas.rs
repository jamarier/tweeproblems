// Structures to support the use of formulas in histories

use lazy_static::lazy_static;
use maplit::hashmap;
use std::collections::HashMap;

type Formulas = HashMap<&'static str, &'static str>;

lazy_static! {
    pub static ref FORMULAS: Formulas = hashmap! {
        "ohm_law_.RI" => " A : I !  ohm : R !   I @ R @ *    V : ",
        "ohm_law_V.I" => " A : I !  V : V !  V @ I @ /    ohm : ",
        "ohm_law_VR." => " ohm : R !    V : V !     V @ R @ /   A : ",

        "parallel" => " ohm : R2 !  ohm : R1 ! R1 @ R2 @ * R1 @ R2 @ + / ohm : "
    };
}
