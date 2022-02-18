/*!-----------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Version: 0.32.0(e1570658ecca35c72429e624c18df24ae4286ef8)
 * Released under the MIT license
 * https://github.com/microsoft/monaco-editor/blob/main/LICENSE.txt
 *-----------------------------------------------------------------------------*/

// src/basic-languages/julia/julia.contribution.ts
import { registerLanguage } from "../_.contribution.js";
registerLanguage({
  id: "julia",
  extensions: [".jl"],
  aliases: ["julia", "Julia"],
  loader: () => {
    if (false) {
      return new Promise((resolve, reject) => {
        __require(["vs/basic-languages/julia/julia"], resolve, reject);
      });
    } else {
      return import("./julia.js");
    }
  }
});
