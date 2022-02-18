/*!-----------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Version: 0.32.0(e1570658ecca35c72429e624c18df24ae4286ef8)
 * Released under the MIT license
 * https://github.com/microsoft/monaco-editor/blob/main/LICENSE.txt
 *-----------------------------------------------------------------------------*/

// src/basic-languages/kotlin/kotlin.contribution.ts
import { registerLanguage } from "../_.contribution.js";
registerLanguage({
  id: "kotlin",
  extensions: [".kt"],
  aliases: ["Kotlin", "kotlin"],
  mimetypes: ["text/x-kotlin-source", "text/x-kotlin"],
  loader: () => {
    if (false) {
      return new Promise((resolve, reject) => {
        __require(["vs/basic-languages/kotlin/kotlin"], resolve, reject);
      });
    } else {
      return import("./kotlin.js");
    }
  }
});
