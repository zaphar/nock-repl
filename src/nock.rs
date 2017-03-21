//! nock implements a nock interpreter.
// Copyright (2017) Jeremy A. Wall.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use parser::{Noun, ParseError, atom};
use std::error;
use std::fmt;
use std::fmt::Display;


make_error!(NockError, "NockError: {}\n");

impl From<ParseError> for NockError {
    fn from(err: ParseError) -> Self {
        Self::new_with_cause("AST Parse Error", Box::new(err))
    }
}

fn slice_to_noun(nouns: &[Noun]) -> Result<Noun, NockError> {
    if nouns.len() > 1 {
        Ok(Noun::Cell(nouns.iter().cloned().collect()))
    } else if nouns.len() == 1 {
        Ok(nouns[0].clone())
    } else {
        Err(NockError::new("!! Nock Empty Cell"))
    }
}

// true means take the head.
// false means take the tail
fn make_tree_path(addr: u64) -> Vec<bool> {
    // 6 is the head of 3
    // 3 is the tail of 1
    // 1 is the whole tree.

    // 13 is the tail of 6
    // 6 is the head of 3
    // 3 is the tail of 1
    // 1 is the whole tree.
    let mut ret = Vec::new();
    let mut next = addr;
    loop {
        ret.push(next % 2 == 0);
        next = next / 2;
        if next <= 1 {
            break;
        }
    }
    // TODO(jwall): if we could not have to reverse that would be nice.
    ret.iter().rev().cloned().collect()
}

fn fas(subj: &Noun, addr: u64) -> Result<Noun, NockError> {
    if addr == 0 {
        return Err(NockError::new("!! Invalid slot address 0"));
    }
    if addr == 1 {
        return Ok(subj.clone());
    }
    if addr == 2 {
        return Ok(try!(subj.head()).clone());
    }
    if addr == 3 {
        return slice_to_noun(try!(subj.tail()));
    }
    let path = make_tree_path(addr);
    let mut subject = subj.clone();
    for take_head in path {
        subject = if take_head {
            try!(subject.head()).clone()
        } else {
            try!(slice_to_noun(try!(subject.tail())))
        }
    }
    Ok(subject)
}

#[cfg(test)]
#[test]
fn test_simple_fas() {
    // /[1 [531 25 99]] is [531 25 99];
    let cases = vec![(cell!(atom(531), atom(25), atom(99)), 1,
                      cell!(atom(531), atom(25), atom(99))),
                     // /[2 [531 25 99]] is 531;
                     (cell!(atom(531), atom(25), atom(99)), 2,
                      atom(531)),
                     // /[3 [531 25 99]] is [25 99];
                     (cell!(atom(531), atom(25), atom(99)), 3,
                      cell!(atom(25), atom(99))),
                     // /[6 [531 25 99]] is 25;
                     (cell!(atom(531), atom(25), atom(99)), 6,
                      atom(25)),
                     (cell!(atom(531), cell!(atom(25), atom(26)), atom(99)), 6,
                      cell!(atom(25), atom(26))),
                     (cell!(atom(531), cell!(atom(25), atom(26)), atom(99)), 12,
                      atom(25)),
                     (cell!(atom(531), cell!(atom(25), atom(26)), atom(99)), 13,
                      atom(26)),
                     ];
    for (subj, addr, expected) in cases {
        assert_eq!(expected, fas(&subj, addr).unwrap());
    }
    // [12 [531 25 99]] crashes
}

#[cfg(test)]
#[test]
#[should_panic]
fn test_fas_crash() {
    let cell = cell!(atom(531), atom(25), atom(99));
    // We expect an error here so we crash.
    fas(&cell, 12).unwrap();
}

// Returns 1 false for an atom and 0 true for a Noun::Cell.
fn wut(noun: Noun) -> Noun {
    match noun {
        Noun::Atom(_) => atom(1),
        Noun::Cell(_) => atom(0),
    }
}

// lus increments a atom but crashes for a Noun::Cell.
fn lus(noun: Noun) -> Result<Noun, NockError> {
    match noun {
        Noun::Atom(a) => Ok(atom(a + 1)),
        Noun::Cell(_) => Err(NockError::new("!! Can't increment a cell")),
    }
}

fn cmp_noun(a: &Noun, b: &[Noun]) -> Noun {
    let falsy = atom(0);
    let truthy = atom(1);
    match a {
        &Noun::Cell(ref list) => {
            if list.len() != b.len() {
                return falsy;
            }
            for (i, n) in list.iter().enumerate() {
                if cmp_noun(n, &b[i..]) == falsy {
                    return falsy;
                }
            }
            return truthy;
        }
        &Noun::Atom(a) => {
            if b.len() != 1 {
                if let Noun::Atom(b) = b[0] {
                    if a == b {
                        return truthy;
                    }
                }
            }
            return falsy;
        }
    }
}

// tis compares a Noun::Cell's head and tail Nouns for equality.
fn tis(noun: Noun) -> Result<Noun, NockError> {
    match noun {
        Noun::Atom(_) => Err(NockError::new("!! Can't compaire Atom like a cell")),
        Noun::Cell(list) => {
            if list.len() >= 2 {
                Ok(cmp_noun(&list[0], &list[1..]))
            } else {
                Err(NockError::new("!! Can't compare a cell of only one Noun"))
            }
        }
    }
}

// evaluates a nock expression of type [subj formula] or [formula] or atom
pub fn eval(noun: Noun) -> Result<Noun, NockError> {
    match &noun {
        &Noun::Atom(_) => nock_internal(&Noun::Atom(0), noun.clone()),
        &Noun::Cell(ref list) => {
            if list.len() >= 2 {
                nock_internal(try!(noun.head()), try!(slice_to_noun(try!(noun.tail()))))
            } else {
                Err(NockError::new("!! Invalid Nock Expression"))
            }
        }
    }
}

// Evaluates a nock formula against a subj.
fn nock_internal(subj: &Noun, formula: Noun) -> Result<Noun, NockError> {
    match formula {
        Noun::Atom(_) => return Err(NockError::new(format!("!! Nock Infinite Loop"))),
        cell => {
            match try!(cell.head()) {
                &Noun::Atom(a) => {
                    // We expect an instruction from 0 to 10
                    match a {
                        0 => {
                            let tail = try!(slice_to_noun(try!(cell.tail())));
                            if let Noun::Atom(b) = tail {
                                return fas(subj, b);
                            } else {
                                return Err(NockError::new(format!("!! not a slot index {}", tail)));
                            }
                        }
                        1 => {
                            return Ok(try!(slice_to_noun(try!(cell.tail()))));
                        }
                        2 => {
                            return Ok(try!(nock_internal(subj,
                                                         try!(slice_to_noun(try!(cell.tail()))))));
                        }
                        3 => {
                            return Ok(wut(try!(nock_internal(subj,
                                                             try!(slice_to_noun(
                                                    try!(cell.tail())))))));
                        }
                        4 => {
                            let tail_noun = try!(slice_to_noun(try!(cell.tail())));
                            if let Noun::Cell(_) = tail_noun {
                                return Ok(try!(lus(try!(nock_internal(subj, tail_noun)))));
                            }
                            return Ok(try!(lus(tail_noun)));
                        }
                        5 => {
                            return Ok(try!(tis(try!(slice_to_noun(try!(cell.tail()))))));
                        }
                        // macros
                        6 => {
                            let tail = try!(cell.tail());
                            if tail.len() < 3 {
                                return Err(NockError::new("!! Need 3 Nouns for macro 6"));
                            }
                            let b = tail[0].clone();
                            let c = tail[1].clone();
                            let d = try!(slice_to_noun(&tail[2..]));
                            // *[a 6 b c d]     *[a 2 [0 1] 2 [1 c d] [1 0] 2 [1 2 3] [1 0] 4 4 b]
                            let formula = cell!(atom(2),
                                                // [0 1]
                                                cell!(atom(0), atom(1)),
                                                // 2
                                                atom(2),
                                                // [1 c d]
                                                cell!(atom(1), c, d),
                                                // [1 0]
                                                cell!(atom(1), atom(0)),
                                                // 2
                                                atom(2),
                                                // [1 2 3]
                                                cell!(atom(1), atom(2), atom(3)),
                                                // [1 0]
                                                cell!(atom(1), atom(0)),
                                                // 4 4 b]
                                                atom(4),
                                                atom(4),
                                                b);
                            return nock_internal(subj, formula);
                        }
                        7 => {
                            let tail = try!(cell.tail());
                            if tail.len() < 2 {
                                return Err(NockError::new("!! Need 3 Nouns for macro 6"));
                            }
                            let b = tail[0].clone();
                            let c = tail[1].clone();
                            // *[a 7 b c] -> *[a 2 b 1 c]
                            let formula = cell!(atom(2), b, atom(1), c);
                            return nock_internal(subj, formula);
                        }
                        8 => {
                            let tail = try!(cell.tail());
                            if tail.len() < 2 {
                                return Err(NockError::new("!! Need 3 Nouns for macro 6"));
                            }
                            let b = tail[0].clone();
                            let c = tail[1].clone();
                            // *[a 8 b c]       *[a 7 [[7 [0 1] b] 0 1] c]
                            let formula = cell!(atom(7),
                                                cell!(cell!(atom(7), cell!(atom(0), atom(1)), b),
                                                      atom(0),
                                                      atom(1)),
                                                c);
                            return nock_internal(subj, formula);
                        }
                        9 => {
                            let tail = try!(cell.tail());
                            if tail.len() < 2 {
                                return Err(NockError::new("!! Need 3 Nouns for macro 6"));
                            }
                            let b = tail[0].clone();
                            let c = tail[1].clone();
                            // *[a 9 b c]       *[a 7 c 2 [0 1] 0 b]
                            let formula =
                                cell!(atom(7), c, atom(2), cell!(atom(0), atom(1)), atom(0), b);
                            return nock_internal(subj, formula);
                        }
                        10 => {
                            let tail = try!(cell.tail());
                            if tail.len() < 2 {
                                return Err(NockError::new("!! Need 3 Nouns for macro 6"));
                            }
                            let b = tail[0].clone();
                            let c = tail[1].clone();
                            match b {
                                Noun::Atom(_) => {
                                    // *[a 10 b c]      *[a c]
                                    // b is discarded.
                                    return nock_internal(subj, c);
                                }
                                Noun::Cell(list) => {
                                    let d = c;
                                    // b is discarded.
                                    let c = try!(slice_to_noun(&list[1..]));
                                    // *[a 10 [b c] d]  *[a 8 c 7 [0 3] d]
                                    let formula =
                                        cell!(atom(8), c, atom(7), cell!(atom(0), atom(3)), d);
                                    return nock_internal(subj, formula);
                                }
                            }
                        }
                        _ => {
                            return Err(NockError::new(format!("!! Unknown Nock instruction {}",
                                                              a)));
                        }
                    }
                }
                head_formula => {
                    let head = try!(nock_internal(subj, head_formula.clone()));
                    let new_formula = try!(slice_to_noun(try!(cell.tail())));
                    let tail_noun = try!(nock_internal(subj, new_formula));
                    return Ok(cell!(head, tail_noun));
                }
            }
        }
    }
}
