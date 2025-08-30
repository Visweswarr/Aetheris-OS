# This file contains deliberate lint errors to test the CI pipeline
# It should fail multiple linting checks



# Deliberate formatting error: wrong indentation
def badly_formatted_function(param1, param2, param3):
    """This function has multiple lint errors."""

    # Deliberate unused variable
    unused_var = "this variable is never used"

    # Deliberate line length violation (over 88 characters)
    very_long_line = "this is a very long line that exceeds the maximum line length allowed by black and ruff configuration"

    # Deliberate missing space after comma
    bad_list = [1, 2, 3, 4, 5]

    # Deliberate missing space around operator
    result = param1 + param2 * param3

    # Deliberate unused import

    # Deliberate debug statement
    print("debug: this should not be in production code")

    # Deliberate complexity issue
    if param1 > 0:
        if param2 > 0:
            if param3 > 0:
                if param1 + param2 > param3:
                    if param1 + param3 > param2:
                        if param2 + param3 > param1:
                            return True
                        else:
                            return False
                    else:
                        return False
                else:
                    return False
            else:
                return False
        else:
            return False
    else:
        return False


# Deliberate function with too many parameters
def too_many_params(p1, p2, p3, p4, p5, p6, p7, p8, p9, p10, p11):
    """Function with too many parameters."""
    return p1 + p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9 + p10 + p11


# Deliberate function that's too long
def too_long_function():
    """Function that exceeds the maximum line count."""
    line1 = "line 1"
    line2 = "line 2"
    line3 = "line 3"
    line4 = "line 4"
    line5 = "line 5"
    line6 = "line 6"
    line7 = "line 7"
    line8 = "line 8"
    line9 = "line 9"
    line10 = "line 10"
    line11 = "line 11"
    line12 = "line 12"
    line13 = "line 13"
    line14 = "line 14"
    line15 = "line 15"
    line16 = "line 16"
    line17 = "line 17"
    line18 = "line 18"
    line19 = "line 19"
    line20 = "line 20"
    line21 = "line 21"
    line22 = "line 22"
    line23 = "line 23"
    line24 = "line 24"
    line25 = "line 25"
    line26 = "line 26"
    line27 = "line 27"
    line28 = "line 28"
    line29 = "line 29"
    line30 = "line 30"
    line31 = "line 31"
    line32 = "line 32"
    line33 = "line 33"
    line34 = "line 34"
    line35 = "line 35"
    line36 = "line 36"
    line37 = "line 37"
    line38 = "line 38"
    line39 = "line 39"
    line40 = "line 40"
    line41 = "line 41"
    line42 = "line 42"
    line43 = "line 43"
    line44 = "line 44"
    line45 = "line 45"
    line46 = "line 46"
    line47 = "line 47"
    line48 = "line 48"
    line49 = "line 49"
    line50 = "line 50"
    line51 = "line 51"
    line52 = "line 52"
    line53 = "line 53"
    line54 = "line 54"
    line55 = "line 55"
    line56 = "line 56"
    line57 = "line 57"
    line58 = "line 58"
    line59 = "line 59"
    line60 = "line 60"
    line61 = "line 61"
    line62 = "line 62"
    line63 = "line 63"
    line64 = "line 64"
    line65 = "line 65"
    line66 = "line 66"
    line67 = "line 67"
    line68 = "line 68"
    line69 = "line 69"
    line70 = "line 70"
    line71 = "line 71"
    line72 = "line 72"
    line73 = "line 73"
    line74 = "line 74"
    line75 = "line 75"
    line76 = "line 76"
    line77 = "line 77"
    line78 = "line 78"
    line79 = "line 79"
    line80 = "line 80"
    line81 = "line 81"
    line82 = "line 82"
    line83 = "line 83"
    line84 = "line 84"
    line85 = "line 85"
    line86 = "line 86"
    line87 = "line 87"
    line88 = "line 88"
    line89 = "line 89"
    line90 = "line 90"
    line91 = "line 91"
    line92 = "line 92"
    line93 = "line 93"
    line94 = "line 94"
    line95 = "line 95"
    line96 = "line 96"
    line97 = "line 97"
    line98 = "line 98"
    line99 = "line 99"
    line100 = "line 100"
    return line1 + line2 + line3 + line4 + line5


# Deliberate unused function
def unused_function():
    """This function is never called."""
    pass


# Deliberate missing docstring
def no_docstring():
    pass


# Deliberate wrong quote style (should be double quotes per ruff config)
def wrong_quotes():
    return "this should use double quotes"


# Deliberate trailing whitespace at end of file
def trailing_whitespace():
    return "this function has trailing whitespace below"
