/*!-----------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Version: 0.32.0(e1570658ecca35c72429e624c18df24ae4286ef8)
 * Released under the MIT license
 * https://github.com/microsoft/monaco-editor/blob/main/LICENSE.txt
 *-----------------------------------------------------------------------------*/
define("vs/basic-languages/cameligo/cameligo", ["require"],(require)=>{
var moduleExports = (() => {
  var __defProp = Object.defineProperty;
  var __markAsModule = (target) => __defProp(target, "__esModule", { value: true });
  var __export = (target, all) => {
    __markAsModule(target);
    for (var name in all)
      __defProp(target, name, { get: all[name], enumerable: true });
  };

  // src/basic-languages/cameligo/cameligo.ts
  var cameligo_exports = {};
  __export(cameligo_exports, {
    conf: () => conf,
    language: () => language
  });
  var conf = {
    comments: {
      lineComment: "//",
      blockComment: ["(*", "*)"]
    },
    brackets: [
      ["{", "}"],
      ["[", "]"],
      ["(", ")"],
      ["<", ">"]
    ],
    autoClosingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: "<", close: ">" },
      { open: "'", close: "'" },
      { open: '"', close: '"' },
      { open: "(*", close: "*)" }
    ],
    surroundingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: "<", close: ">" },
      { open: "'", close: "'" },
      { open: '"', close: '"' },
      { open: "(*", close: "*)" }
    ]
  };
  var language = {
    defaultToken: "",
    tokenPostfix: ".cameligo",
    ignoreCase: true,
    brackets: [
      { open: "{", close: "}", token: "delimiter.curly" },
      { open: "[", close: "]", token: "delimiter.square" },
      { open: "(", close: ")", token: "delimiter.parenthesis" },
      { open: "<", close: ">", token: "delimiter.angle" }
    ],
    keywords: [
      "abs",
      "assert",
      "block",
      "Bytes",
      "case",
      "Crypto",
      "Current",
      "else",
      "failwith",
      "false",
      "for",
      "fun",
      "if",
      "in",
      "let",
      "let%entry",
      "let%init",
      "List",
      "list",
      "Map",
      "map",
      "match",
      "match%nat",
      "mod",
      "not",
      "operation",
      "Operation",
      "of",
      "record",
      "Set",
      "set",
      "sender",
      "skip",
      "source",
      "String",
      "then",
      "to",
      "true",
      "type",
      "with"
    ],
    typeKeywords: ["int", "unit", "string", "tz", "nat", "bool"],
    operators: [
      "=",
      ">",
      "<",
      "<=",
      ">=",
      "<>",
      ":",
      ":=",
      "and",
      "mod",
      "or",
      "+",
      "-",
      "*",
      "/",
      "@",
      "&",
      "^",
      "%",
      "->",
      "<-",
      "&&",
      "||"
    ],
    symbols: /[=><:@\^&|+\-*\/\^%]+/,
    tokenizer: {
      root: [
        [
          /[a-zA-Z_][\w]*/,
          {
            cases: {
              "@keywords": { token: "keyword.$0" },
              "@default": "identifier"
            }
          }
        ],
        { include: "@whitespace" },
        [/[{}()\[\]]/, "@brackets"],
        [/[<>](?!@symbols)/, "@brackets"],
        [
          /@symbols/,
          {
            cases: {
              "@operators": "delimiter",
              "@default": ""
            }
          }
        ],
        [/\d*\.\d+([eE][\-+]?\d+)?/, "number.float"],
        [/\$[0-9a-fA-F]{1,16}/, "number.hex"],
        [/\d+/, "number"],
        [/[;,.]/, "delimiter"],
        [/'([^'\\]|\\.)*$/, "string.invalid"],
        [/'/, "string", "@string"],
        [/'[^\\']'/, "string"],
        [/'/, "string.invalid"],
        [/\#\d+/, "string"]
      ],
      comment: [
        [/[^\(\*]+/, "comment"],
        [/\*\)/, "comment", "@pop"],
        [/\(\*/, "comment"]
      ],
      string: [
        [/[^\\']+/, "string"],
        [/\\./, "string.escape.invalid"],
        [/'/, { token: "string.quote", bracket: "@close", next: "@pop" }]
      ],
      whitespace: [
        [/[ \t\r\n]+/, "white"],
        [/\(\*/, "comment", "@comment"],
        [/\/\/.*$/, "comment"]
      ]
    }
  };
  return cameligo_exports;
})();
return moduleExports;
});
