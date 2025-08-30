load("@aspect_rules_ts//ts:defs.bzl", "ts_project")
load("@aspect_rules_js//js:defs.bzl", "js_binary")

ts_project(
    name = "ngfs_path_check_ts",
    srcs = ["path_check.ts"],
    declaration = True,
    tsconfig = "//tooling/ts:tsconfig.json",
    visibility = ["//visibility:public"],
)

js_binary(
    name = "ngfs_path_check",
    data = [":ngfs_path_check_ts"],
    entry_point = "path_check.js",
    visibility = ["//visibility:public"],
)
