// This file contains deliberate lint errors to test the CI pipeline
// It should fail multiple linting checks

import * as os from 'os';
import * as fs from 'fs';
import * as path from 'path';

// Deliberate unused import
import * as crypto from 'crypto';

// Deliberate formatting error: wrong indentation
function badlyFormattedFunction(param1:number,param2:string,param3:boolean){
    // Deliberate unused variable
    const unusedVar = "this variable is never used";

    // Deliberate line length violation (over 100 characters)
    const veryLongLine = "this is a very long line that exceeds the maximum line length allowed by prettier and eslint configuration and should trigger a linting error";

    // Deliberate missing space after comma
    const badArray = [1,2,3,4,5];

    // Deliberate missing space around operator
    const result=param1+2*3;

    // Deliberate missing semicolon
    const missingSemicolon = "this line is missing a semicolon"

    // Deliberate wrong quote style (should be single quotes per eslint config)
    const wrongQuotes = "this should use single quotes";

    // Deliberate debug statement
    console.log("debug: this should not be in production code");

    // Deliberate complexity issue
    if (param1 > 0) {
        if (param2.length > 0) {
            if (param3 === true) {
                if (param1 + param2.length > 10) {
                    if (param1 + 5 > param2.length) {
                        if (param2.length + 5 > param1) {
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
function tooManyParams(p1: number, p2: number, p3: number, p4: number, p5: number, p6: number, p7: number, p8: number, p9: number, p10: number, p11: number): number {
    return p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 + p10 + p11;
}

// Deliberate function that's too long
function tooLongFunction(): string {
    const line1 = "line 1";
    const line2 = "line 2";
    const line3 = "line 3";
    const line4 = "line 4";
    const line5 = "line 5";
    const line6 = "line 6";
    const line7 = "line 7";
    const line8 = "line 8";
    const line9 = "line 9";
    const line10 = "line 10";
    const line11 = "line 11";
    const line12 = "line 12";
    const line13 = "line 13";
    const line14 = "line 14";
    const line15 = "line 15";
    const line16 = "line 16";
    const line17 = "line 17";
    const line18 = "line 18";
    const line19 = "line 19";
    const line20 = "line 20";
    const line21 = "line 21";
    const line22 = "line 22";
    const line23 = "line 23";
    const line24 = "line 24";
    const line25 = "line 25";
    const line26 = "line 26";
    const line27 = "line 27";
    const line28 = "line 28";
    const line29 = "line 29";
    const line30 = "line 30";
    const line31 = "line 31";
    const line32 = "line 32";
    const line33 = "line 33";
    const line34 = "line 34";
    const line35 = "line 35";
    const line36 = "line 36";
    const line37 = "line 37";
    const line38 = "line 38";
    const line39 = "line 39";
    const line40 = "line 40";
    const line41 = "line 41";
    const line42 = "line 42";
    const line43 = "line 43";
    const line44 = "line 44";
    const line45 = "line 45";
    const line46 = "line 46";
    const line47 = "line 47";
    const line48 = "line 48";
    const line49 = "line 49";
    const line50 = "line 50";
    return line1 + line2 + line3 + line4 + line5 + line6 + line7 + line8 + line9 + line10 + line11 + line12 + line13 + line14 + line15 + line16 + line17 + line18 + line19 + line20 + line21 + line22 + line23 + line24 + line25 + line26 + line27 + line28 + line29 + line30 + line31 + line32 + line33 + line34 + line35 + line36 + line37 + line38 + line39 + line40 + line41 + line42 + line43 + line44 + line45 + line46 + line47 + line48 + line49 + line50;
}

// Deliberate unused function
function unusedFunction(): void {
    console.log("This function is never called");
}

// Deliberate missing return type
function noReturnType() {
    return "this function is missing a return type annotation";
}

// Deliberate any type usage
function usesAnyType(param: any): any {
    return param;
}

// Deliberate trailing whitespace at end of file
function trailingWhitespace(): string {
    return "this function has trailing whitespace below";
