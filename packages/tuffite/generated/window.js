// Generated from ../../framework/shell_api/window.d.ts. Do not edit.

export const entries = [
  [
    "window.current",
    [],
    {
      "fields": [
        [
          "id",
          {
            "kind": "bigint"
          }
        ],
        [
          "label",
          {
            "kind": "string"
          }
        ],
        [
          "title",
          {
            "kind": "string"
          }
        ],
        [
          "url",
          {
            "kind": "string"
          }
        ],
        [
          "bounds",
          {
            "fields": [
              [
                "x",
                {
                  "kind": "number"
                }
              ],
              [
                "y",
                {
                  "kind": "number"
                }
              ],
              [
                "width",
                {
                  "kind": "number"
                }
              ],
              [
                "height",
                {
                  "kind": "number"
                }
              ]
            ],
            "kind": "object"
          }
        ],
        [
          "focused",
          {
            "kind": "boolean"
          }
        ],
        [
          "minimized",
          {
            "kind": "boolean"
          }
        ],
        [
          "maximized",
          {
            "kind": "boolean"
          }
        ],
        [
          "fullscreen",
          {
            "kind": "boolean"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "window.list",
    [],
    {
      "item": {
        "fields": [
          [
            "id",
            {
              "kind": "bigint"
            }
          ],
          [
            "label",
            {
              "kind": "string"
            }
          ],
          [
            "title",
            {
              "kind": "string"
            }
          ],
          [
            "url",
            {
              "kind": "string"
            }
          ],
          [
            "bounds",
            {
              "fields": [
                [
                  "x",
                  {
                    "kind": "number"
                  }
                ],
                [
                  "y",
                  {
                    "kind": "number"
                  }
                ],
                [
                  "width",
                  {
                    "kind": "number"
                  }
                ],
                [
                  "height",
                  {
                    "kind": "number"
                  }
                ]
              ],
              "kind": "object"
            }
          ],
          [
            "focused",
            {
              "kind": "boolean"
            }
          ],
          [
            "minimized",
            {
              "kind": "boolean"
            }
          ],
          [
            "maximized",
            {
              "kind": "boolean"
            }
          ],
          [
            "fullscreen",
            {
              "kind": "boolean"
            }
          ]
        ],
        "kind": "object"
      },
      "kind": "array"
    }
  ],
  [
    "window.create",
    [
      {
        "kind": "optional",
        "value": {
          "fields": [
            [
              "label",
              {
                "kind": "optional",
                "value": {
                  "kind": "string"
                }
              }
            ],
            [
              "url",
              {
                "kind": "optional",
                "value": {
                  "kind": "string"
                }
              }
            ],
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
              "bounds",
              {
                "kind": "optional",
                "value": {
                  "fields": [
                    [
                      "x",
                      {
                        "kind": "number"
                      }
                    ],
                    [
                      "y",
                      {
                        "kind": "number"
                      }
                    ],
                    [
                      "width",
                      {
                        "kind": "number"
                      }
                    ],
                    [
                      "height",
                      {
                        "kind": "number"
                      }
                    ]
                  ],
                  "kind": "object"
                }
              }
            ],
            [
              "focused",
              {
                "kind": "optional",
                "value": {
                  "kind": "boolean"
                }
              }
            ]
          ],
          "kind": "object"
        }
      }
    ],
    {
      "fields": [
        [
          "id",
          {
            "kind": "bigint"
          }
        ],
        [
          "label",
          {
            "kind": "string"
          }
        ],
        [
          "title",
          {
            "kind": "string"
          }
        ],
        [
          "url",
          {
            "kind": "string"
          }
        ],
        [
          "bounds",
          {
            "fields": [
              [
                "x",
                {
                  "kind": "number"
                }
              ],
              [
                "y",
                {
                  "kind": "number"
                }
              ],
              [
                "width",
                {
                  "kind": "number"
                }
              ],
              [
                "height",
                {
                  "kind": "number"
                }
              ]
            ],
            "kind": "object"
          }
        ],
        [
          "focused",
          {
            "kind": "boolean"
          }
        ],
        [
          "minimized",
          {
            "kind": "boolean"
          }
        ],
        [
          "maximized",
          {
            "kind": "boolean"
          }
        ],
        [
          "fullscreen",
          {
            "kind": "boolean"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "window.focus",
    [
      {
        "kind": "optional",
        "value": {
          "kind": "bigint"
        }
      }
    ],
    {
      "fields": [
        [
          "id",
          {
            "kind": "bigint"
          }
        ],
        [
          "label",
          {
            "kind": "string"
          }
        ],
        [
          "title",
          {
            "kind": "string"
          }
        ],
        [
          "url",
          {
            "kind": "string"
          }
        ],
        [
          "bounds",
          {
            "fields": [
              [
                "x",
                {
                  "kind": "number"
                }
              ],
              [
                "y",
                {
                  "kind": "number"
                }
              ],
              [
                "width",
                {
                  "kind": "number"
                }
              ],
              [
                "height",
                {
                  "kind": "number"
                }
              ]
            ],
            "kind": "object"
          }
        ],
        [
          "focused",
          {
            "kind": "boolean"
          }
        ],
        [
          "minimized",
          {
            "kind": "boolean"
          }
        ],
        [
          "maximized",
          {
            "kind": "boolean"
          }
        ],
        [
          "fullscreen",
          {
            "kind": "boolean"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "window.minimize",
    [
      {
        "kind": "optional",
        "value": {
          "kind": "bigint"
        }
      }
    ],
    {
      "fields": [
        [
          "id",
          {
            "kind": "bigint"
          }
        ],
        [
          "label",
          {
            "kind": "string"
          }
        ],
        [
          "title",
          {
            "kind": "string"
          }
        ],
        [
          "url",
          {
            "kind": "string"
          }
        ],
        [
          "bounds",
          {
            "fields": [
              [
                "x",
                {
                  "kind": "number"
                }
              ],
              [
                "y",
                {
                  "kind": "number"
                }
              ],
              [
                "width",
                {
                  "kind": "number"
                }
              ],
              [
                "height",
                {
                  "kind": "number"
                }
              ]
            ],
            "kind": "object"
          }
        ],
        [
          "focused",
          {
            "kind": "boolean"
          }
        ],
        [
          "minimized",
          {
            "kind": "boolean"
          }
        ],
        [
          "maximized",
          {
            "kind": "boolean"
          }
        ],
        [
          "fullscreen",
          {
            "kind": "boolean"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "window.maximize",
    [
      {
        "kind": "optional",
        "value": {
          "kind": "bigint"
        }
      }
    ],
    {
      "fields": [
        [
          "id",
          {
            "kind": "bigint"
          }
        ],
        [
          "label",
          {
            "kind": "string"
          }
        ],
        [
          "title",
          {
            "kind": "string"
          }
        ],
        [
          "url",
          {
            "kind": "string"
          }
        ],
        [
          "bounds",
          {
            "fields": [
              [
                "x",
                {
                  "kind": "number"
                }
              ],
              [
                "y",
                {
                  "kind": "number"
                }
              ],
              [
                "width",
                {
                  "kind": "number"
                }
              ],
              [
                "height",
                {
                  "kind": "number"
                }
              ]
            ],
            "kind": "object"
          }
        ],
        [
          "focused",
          {
            "kind": "boolean"
          }
        ],
        [
          "minimized",
          {
            "kind": "boolean"
          }
        ],
        [
          "maximized",
          {
            "kind": "boolean"
          }
        ],
        [
          "fullscreen",
          {
            "kind": "boolean"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "window.restore",
    [
      {
        "kind": "optional",
        "value": {
          "kind": "bigint"
        }
      }
    ],
    {
      "fields": [
        [
          "id",
          {
            "kind": "bigint"
          }
        ],
        [
          "label",
          {
            "kind": "string"
          }
        ],
        [
          "title",
          {
            "kind": "string"
          }
        ],
        [
          "url",
          {
            "kind": "string"
          }
        ],
        [
          "bounds",
          {
            "fields": [
              [
                "x",
                {
                  "kind": "number"
                }
              ],
              [
                "y",
                {
                  "kind": "number"
                }
              ],
              [
                "width",
                {
                  "kind": "number"
                }
              ],
              [
                "height",
                {
                  "kind": "number"
                }
              ]
            ],
            "kind": "object"
          }
        ],
        [
          "focused",
          {
            "kind": "boolean"
          }
        ],
        [
          "minimized",
          {
            "kind": "boolean"
          }
        ],
        [
          "maximized",
          {
            "kind": "boolean"
          }
        ],
        [
          "fullscreen",
          {
            "kind": "boolean"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "window.setFullscreen",
    [
      {
        "kind": "boolean"
      },
      {
        "kind": "optional",
        "value": {
          "kind": "bigint"
        }
      }
    ],
    {
      "fields": [
        [
          "id",
          {
            "kind": "bigint"
          }
        ],
        [
          "label",
          {
            "kind": "string"
          }
        ],
        [
          "title",
          {
            "kind": "string"
          }
        ],
        [
          "url",
          {
            "kind": "string"
          }
        ],
        [
          "bounds",
          {
            "fields": [
              [
                "x",
                {
                  "kind": "number"
                }
              ],
              [
                "y",
                {
                  "kind": "number"
                }
              ],
              [
                "width",
                {
                  "kind": "number"
                }
              ],
              [
                "height",
                {
                  "kind": "number"
                }
              ]
            ],
            "kind": "object"
          }
        ],
        [
          "focused",
          {
            "kind": "boolean"
          }
        ],
        [
          "minimized",
          {
            "kind": "boolean"
          }
        ],
        [
          "maximized",
          {
            "kind": "boolean"
          }
        ],
        [
          "fullscreen",
          {
            "kind": "boolean"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "window.close",
    [
      {
        "kind": "optional",
        "value": {
          "kind": "bigint"
        }
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "window.setTitle",
    [
      {
        "kind": "string"
      },
      {
        "kind": "optional",
        "value": {
          "kind": "bigint"
        }
      }
    ],
    {
      "fields": [
        [
          "id",
          {
            "kind": "bigint"
          }
        ],
        [
          "label",
          {
            "kind": "string"
          }
        ],
        [
          "title",
          {
            "kind": "string"
          }
        ],
        [
          "url",
          {
            "kind": "string"
          }
        ],
        [
          "bounds",
          {
            "fields": [
              [
                "x",
                {
                  "kind": "number"
                }
              ],
              [
                "y",
                {
                  "kind": "number"
                }
              ],
              [
                "width",
                {
                  "kind": "number"
                }
              ],
              [
                "height",
                {
                  "kind": "number"
                }
              ]
            ],
            "kind": "object"
          }
        ],
        [
          "focused",
          {
            "kind": "boolean"
          }
        ],
        [
          "minimized",
          {
            "kind": "boolean"
          }
        ],
        [
          "maximized",
          {
            "kind": "boolean"
          }
        ],
        [
          "fullscreen",
          {
            "kind": "boolean"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "window.getBounds",
    [
      {
        "kind": "optional",
        "value": {
          "kind": "bigint"
        }
      }
    ],
    {
      "fields": [
        [
          "x",
          {
            "kind": "number"
          }
        ],
        [
          "y",
          {
            "kind": "number"
          }
        ],
        [
          "width",
          {
            "kind": "number"
          }
        ],
        [
          "height",
          {
            "kind": "number"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "window.setBounds",
    [
      {
        "fields": [
          [
            "x",
            {
              "kind": "number"
            }
          ],
          [
            "y",
            {
              "kind": "number"
            }
          ],
          [
            "width",
            {
              "kind": "number"
            }
          ],
          [
            "height",
            {
              "kind": "number"
            }
          ]
        ],
        "kind": "object"
      },
      {
        "kind": "optional",
        "value": {
          "kind": "bigint"
        }
      }
    ],
    {
      "fields": [
        [
          "id",
          {
            "kind": "bigint"
          }
        ],
        [
          "label",
          {
            "kind": "string"
          }
        ],
        [
          "title",
          {
            "kind": "string"
          }
        ],
        [
          "url",
          {
            "kind": "string"
          }
        ],
        [
          "bounds",
          {
            "fields": [
              [
                "x",
                {
                  "kind": "number"
                }
              ],
              [
                "y",
                {
                  "kind": "number"
                }
              ],
              [
                "width",
                {
                  "kind": "number"
                }
              ],
              [
                "height",
                {
                  "kind": "number"
                }
              ]
            ],
            "kind": "object"
          }
        ],
        [
          "focused",
          {
            "kind": "boolean"
          }
        ],
        [
          "minimized",
          {
            "kind": "boolean"
          }
        ],
        [
          "maximized",
          {
            "kind": "boolean"
          }
        ],
        [
          "fullscreen",
          {
            "kind": "boolean"
          }
        ]
      ],
      "kind": "object"
    }
  ]
];
export const constants = [
];
