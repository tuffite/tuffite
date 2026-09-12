// Generated from ../../framework/shell_api/dialog.d.ts. Do not edit.

export const entries = [
  [
    "dialog.openFile",
    [
      {
        "kind": "optional",
        "value": {
          "fields": [
            [
              "title",
              {
                "kind": "optional",
                "value": {
                  "kind": "string"
                }
              }
            ],
            [
              "defaultPath",
              {
                "kind": "optional",
                "value": {
                  "kind": "string"
                }
              }
            ],
            [
              "extensions",
              {
                "kind": "optional",
                "value": {
                  "item": {
                    "item": {
                      "kind": "string"
                    },
                    "kind": "array"
                  },
                  "kind": "array"
                }
              }
            ]
          ],
          "kind": "object"
        }
      }
    ],
    {
      "kind": "optional",
      "value": {
        "kind": "string"
      }
    }
  ],
  [
    "dialog.openDirectory",
    [
      {
        "kind": "optional",
        "value": {
          "fields": [
            [
              "title",
              {
                "kind": "optional",
                "value": {
                  "kind": "string"
                }
              }
            ]
          ],
          "kind": "object"
        }
      }
    ],
    {
      "kind": "optional",
      "value": {
        "kind": "string"
      }
    }
  ]
];
export const constants = [
];
