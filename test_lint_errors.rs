// This file contains deliberate lint errors to test the CI pipeline
// It should fail multiple linting checks

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

// Deliberate unused import
use std::path::Path;

// Deliberate formatting error: wrong indentation
fn badly_formatted_function(  param1:i32,param2:String,param3:bool)->bool{
    // Deliberate unused variable
    let unused_var = "this variable is never used";

    // Deliberate line length violation (over 100 characters)
    let very_long_line = "this is a very long line that exceeds the maximum line length allowed by rustfmt configuration and should trigger a formatting error";

    // Deliberate missing space after comma
    let bad_array = [1,2,3,4,5];

    // Deliberate missing space around operator
    let result=param1+2*3;

    // Deliberate debug statement
    println!("debug: this should not be in production code");

    // Deliberate complexity issue
    if param1 > 0 {
        if param2.len() > 0 {
            if param3 {
                if param1 + param2.len() as i32 > 10 {
                    if param1 + 5 > param2.len() as i32 {
                        if param2.len() as i32 + 5 > param1 {
                            return true;
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    }
}

// Deliberate function with too many parameters
fn too_many_params(p1: i32, p2: i32, p3: i32, p4: i32, p5: i32, p6: i32, p7: i32, p8: i32, p9: i32, p10: i32, p11: i32) -> i32 {
    p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 + p10 + p11
}

// Deliberate unused function
fn unused_function() {
    println!("This function is never called");
}

// Deliberate missing return type
fn no_return_type() {
    println!("This function is missing a return type annotation");
}

// Deliberate trailing whitespace at end of file
fn trailing_whitespace() -> &'static str {
    "this function has trailing whitespace below"
