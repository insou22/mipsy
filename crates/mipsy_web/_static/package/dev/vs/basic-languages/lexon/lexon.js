/*!-----------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Version: 0.32.0(e1570658ecca35c72429e624c18df24ae4286ef8)
 * Released under the MIT license
 * https://github.com/microsoft/monaco-editor/blob/main/LICENSE.txt
 *-----------------------------------------------------------------------------*/
define("vs/basic-languages/lexon/lexon", ["require"],(require)=>{
var moduleExports = (() => {
  var __defProp = Object.defineProperty;
  var __markAsModule = (target) => __defProp(target, "__esModule", { value: true });
  var __export = (target, all) => {
    __markAsModule(target);
    for (var name in all)
      __defProp(target, name, { get: all[name], enumerable: true });
  };

  // src/basic-languages/lexon/lexon.ts
  var lexon_exports = {};
  __export(lexon_exports, {
    conf: () => conf,
    language: () => language
  });
  var conf = {
    comments: {
      lineComment: "COMMENT"
    },
    brackets: [["(", ")"]],
    autoClosingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: '"', close: '"' },
      { open: ":", close: "." }
    ],
    surroundingPairs: [
      { open: "{", close: "}" },
      { open: "[", close: "]" },
      { open: "(", close: ")" },
      { open: "`", close: "`" },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
      { open: ":", close: "." }
    ],
    folding: {
      markers: {
        start: new RegExp("^\\s*(::\\s*|COMMENT\\s+)#region"),
        end: new RegExp("^\\s*(::\\s*|COMMENT\\s+)#endregion")
      }
    }
  };
  var language = {
    tokenPostfix: ".lexon",
    ignoreCase: true,
    keywords: [
      "lexon",
      "lex",
      "clause",
      "terms",
      "contracts",
      "may",
      "pay",
      "pays",
      "appoints",
      "into",
      "to"
    ],
    typeKeywords: ["amount", "person", "key", "time", "date", "asset", "text"],
    operators: [
      "less",
      "greater",
      "equal",
      "le",
      "gt",
      "or",
      "and",
      "add",
      "added",
      "subtract",
      "subtracted",
      "multiply",
      "multiplied",
      "times",
      "divide",
      "divided",
      "is",
      "be",
      "certified"
    ],
    symbols: /[=><!~?:&|+\-*\/\^%]+/,
    tokenizer: {
      root: [
        [/^(\s*)(comment:?(?:\s.*|))$/, ["", "comment"]],
        [
          /"/,
          {
            token: "identifier.quote",
            bracket: "@open",
            next: "@quoted_identifier"
          }
        ],
        [
          "LEX$",
          {
            token: "keyword",
            bracket: "@open",
            next: "@identifier_until_period"
          }
        ],
        ["LEXON", { token: "keyword", bracket: "@open", next: "@semver" }],
        [
          ":",
          {
            token: "delimiter",
            bracket: "@open",
            next: "@identifier_until_period"
          }
        ],
        [
          /[a-z_$][\w$]*/,
          {
            cases: {
              "@operators": "operator",
              "@typeKeywords": "keyword.type",
              "@keywords": "keyword",
              "@default": "identifier"
            }
          }
        ],
        { include: "@whitespace" },
        [/[{}()\[\]]/, "@brackets"],
        [/[<>](?!@symbols)/, "@brackets"],
        [/@symbols/, "delimiter"],
        [/\d*\.\d*\.\d*/, "number.semver"],
        [/\d*\.\d+([eE][\-+]?\d+)?/, "number.float"],
        [/0[xX][0-9a-fA-F]+/, "number.hex"],
        [/\d+/, "number"],
        [/[;,.]/, "delimiter"]
      ],
      quoted_identifier: [
        [/[^\\"]+/, "identifier"],
        [/"/, { token: "identifier.quote", bracket: "@close", next: "@pop" }]
      ],
      space_identifier_until_period: [
        [":", "delimiter"],
        [" ", { token: "white", next: "@identifier_rest" }]
      ],
      identifier_until_period: [
        { include: "@whitespace" },
        [":", { token: "delimiter", next: "@identifier_rest" }],
        [/[^\\.]+/, "identifier"],
        [/\./, { token: "delimiter", bracket: "@close", next: "@pop" }]
      ],
      identifier_rest: [
        [/[^\\.]+/, "identifier"],
        [/\./, { token: "delimiter", bracket: "@close", next: "@pop" }]
      ],
      semver: [
        { include: "@whitespace" },
        [":", "delimiter"],
        [/\d*\.\d*\.\d*/, { token: "number.semver", bracket: "@close", next: "@pop" }]
      ],
      whitespace: [[/[ \t\r\n]+/, "white"]]
    }
  };
  return lexon_exports;
})();
return moduleExports;
});
