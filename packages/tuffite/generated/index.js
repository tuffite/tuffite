// Generated from ../../framework/shell_api/common.d.ts, ../../framework/shell_api/dialog.d.ts, ../../framework/shell_api/path.d.ts, ../../framework/shell_api/remoting.d.ts, ../../framework/shell_api/fs.d.ts, ../../framework/shell_api/storage.d.ts, ../../framework/shell_api/log.d.ts, ../../framework/shell_api/network.d.ts, ../../framework/shell_api/utility.d.ts, ../../framework/shell_api/window.d.ts. Do not edit.
import { createFrameworkApi } from '/tuffite-runtime.js';

const api = createFrameworkApi([
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
  ],
  [
    "path.directories",
    [],
    {
      "fields": [
        [
          "resource",
          {
            "kind": "string"
          }
        ],
        [
          "data",
          {
            "kind": "string"
          }
        ],
        [
          "cache",
          {
            "kind": "string"
          }
        ],
        [
          "temp",
          {
            "kind": "string"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "path.resolve",
    [
      {
        "kind": "string"
      },
      {
        "kind": "optional",
        "value": {
          "kind": "string"
        }
      }
    ],
    {
      "kind": "string"
    }
  ],
  [
    "remoting.capabilities",
    [],
    {
      "fields": [
        [
          "available",
          {
            "kind": "boolean"
          }
        ],
        [
          "protocol",
          {
            "kind": "string"
          }
        ],
        [
          "videoCodecs",
          {
            "item": {
              "kind": "string"
            },
            "kind": "array"
          }
        ],
        [
          "input",
          {
            "kind": "boolean"
          }
        ],
        [
          "clipboard",
          {
            "kind": "boolean"
          }
        ],
        [
          "reconnect",
          {
            "kind": "boolean"
          }
        ],
        [
          "snapshot",
          {
            "kind": "boolean"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "remoting.start",
    [
      {
        "fields": [
          [
            "authKey",
            {
              "kind": "string"
            }
          ],
          [
            "iceServers",
            {
              "item": {
                "fields": [
                  [
                    "urls",
                    {
                      "item": {
                        "kind": "string"
                      },
                      "kind": "array"
                    }
                  ],
                  [
                    "username",
                    {
                      "kind": "optional",
                      "value": {
                        "kind": "string"
                      }
                    }
                  ],
                  [
                    "credential",
                    {
                      "kind": "optional",
                      "value": {
                        "kind": "string"
                      }
                    }
                  ]
                ],
                "kind": "object"
              },
              "kind": "array"
            }
          ],
          [
            "maxBitrateKbps",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "targetFramerate",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "screenId",
            {
              "kind": "optional",
              "value": {
                "kind": "bigint"
              }
            }
          ]
        ],
        "kind": "object"
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
          "state",
          {
            "kind": "string"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "remoting.processSignal",
    [
      {
        "kind": "bigint"
      },
      {
        "fields": [
          [
            "sequence",
            {
              "kind": "bigint"
            }
          ],
          [
            "description",
            {
              "kind": "optional",
              "value": {
                "fields": [
                  [
                    "type",
                    {
                      "kind": "string"
                    }
                  ],
                  [
                    "sdp",
                    {
                      "kind": "string"
                    }
                  ]
                ],
                "kind": "object"
              }
            }
          ],
          [
            "candidates",
            {
              "item": {
                "fields": [
                  [
                    "mid",
                    {
                      "kind": "string"
                    }
                  ],
                  [
                    "mLineIndex",
                    {
                      "kind": "number"
                    }
                  ],
                  [
                    "candidate",
                    {
                      "kind": "string"
                    }
                  ]
                ],
                "kind": "object"
              },
              "kind": "array"
            }
          ]
        ],
        "kind": "object"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "remoting.takeSignals",
    [
      {
        "kind": "bigint"
      },
      {
        "kind": "optional",
        "value": {
          "kind": "bigint"
        }
      }
    ],
    {
      "item": {
        "fields": [
          [
            "sequence",
            {
              "kind": "bigint"
            }
          ],
          [
            "description",
            {
              "kind": "optional",
              "value": {
                "fields": [
                  [
                    "type",
                    {
                      "kind": "string"
                    }
                  ],
                  [
                    "sdp",
                    {
                      "kind": "string"
                    }
                  ]
                ],
                "kind": "object"
              }
            }
          ],
          [
            "candidates",
            {
              "item": {
                "fields": [
                  [
                    "mid",
                    {
                      "kind": "string"
                    }
                  ],
                  [
                    "mLineIndex",
                    {
                      "kind": "number"
                    }
                  ],
                  [
                    "candidate",
                    {
                      "kind": "string"
                    }
                  ]
                ],
                "kind": "object"
              },
              "kind": "array"
            }
          ]
        ],
        "kind": "object"
      },
      "kind": "array"
    }
  ],
  [
    "remoting.sendInput",
    [
      {
        "kind": "bigint"
      },
      {
        "fields": [
          [
            "kind",
            {
              "kind": "string"
            }
          ],
          [
            "pressed",
            {
              "kind": "optional",
              "value": {
                "kind": "boolean"
              }
            }
          ],
          [
            "usbKeycode",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "text",
            {
              "kind": "optional",
              "value": {
                "kind": "string"
              }
            }
          ],
          [
            "x",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "y",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "button",
            {
              "kind": "optional",
              "value": {
                "kind": "string"
              }
            }
          ],
          [
            "buttonDown",
            {
              "kind": "optional",
              "value": {
                "kind": "boolean"
              }
            }
          ],
          [
            "wheelDeltaX",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "wheelDeltaY",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "wheelTicksX",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "wheelTicksY",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "clipboardMimeType",
            {
              "kind": "optional",
              "value": {
                "kind": "string"
              }
            }
          ],
          [
            "clipboardData",
            {
              "kind": "optional",
              "value": {
                "kind": "bytes"
              }
            }
          ]
        ],
        "kind": "object"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "remoting.stop",
    [
      {
        "kind": "bigint"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "remoting.stats",
    [
      {
        "kind": "bigint"
      }
    ],
    {
      "fields": [
        [
          "state",
          {
            "kind": "string"
          }
        ],
        [
          "route",
          {
            "kind": "string"
          }
        ],
        [
          "transport",
          {
            "kind": "string"
          }
        ],
        [
          "latencyMs",
          {
            "kind": "number"
          }
        ],
        [
          "jitterMs",
          {
            "kind": "number"
          }
        ],
        [
          "availableBandwidthKbps",
          {
            "kind": "number"
          }
        ],
        [
          "packetLossPercent",
          {
            "kind": "number"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "remoting.capture",
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
        ],
        [
          "mimeType",
          {
            "kind": "string"
          }
        ],
        [
          "data",
          {
            "kind": "bytes"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "remoting.injectInput",
    [
      {
        "fields": [
          [
            "kind",
            {
              "kind": "string"
            }
          ],
          [
            "pressed",
            {
              "kind": "optional",
              "value": {
                "kind": "boolean"
              }
            }
          ],
          [
            "usbKeycode",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "text",
            {
              "kind": "optional",
              "value": {
                "kind": "string"
              }
            }
          ],
          [
            "x",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "y",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "button",
            {
              "kind": "optional",
              "value": {
                "kind": "string"
              }
            }
          ],
          [
            "buttonDown",
            {
              "kind": "optional",
              "value": {
                "kind": "boolean"
              }
            }
          ],
          [
            "wheelDeltaX",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "wheelDeltaY",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "wheelTicksX",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "wheelTicksY",
            {
              "kind": "optional",
              "value": {
                "kind": "number"
              }
            }
          ],
          [
            "clipboardMimeType",
            {
              "kind": "optional",
              "value": {
                "kind": "string"
              }
            }
          ],
          [
            "clipboardData",
            {
              "kind": "optional",
              "value": {
                "kind": "bytes"
              }
            }
          ]
        ],
        "kind": "object"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "fs.readFile",
    [
      {
        "kind": "string"
      },
      {
        "kind": "string"
      }
    ],
    {
      "kind": "bytes"
    }
  ],
  [
    "fs.writeFile",
    [
      {
        "kind": "string"
      },
      {
        "kind": "string"
      },
      {
        "kind": "bytes"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "fs.exists",
    [
      {
        "kind": "string"
      },
      {
        "kind": "string"
      }
    ],
    {
      "kind": "boolean"
    }
  ],
  [
    "fs.createDir",
    [
      {
        "kind": "string"
      },
      {
        "kind": "string"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "fs.removeFile",
    [
      {
        "kind": "string"
      },
      {
        "kind": "string"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "storage.get",
    [
      {
        "kind": "string"
      }
    ],
    {
      "kind": "optional",
      "value": {
        "kind": "bytes"
      }
    }
  ],
  [
    "storage.set",
    [
      {
        "kind": "string"
      },
      {
        "kind": "bytes"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "storage.remove",
    [
      {
        "kind": "string"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "storage.clear",
    [],
    {
      "kind": "void"
    }
  ],
  [
    "log.trace",
    [
      {
        "kind": "string"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "log.debug",
    [
      {
        "kind": "string"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "log.info",
    [
      {
        "kind": "string"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "log.warn",
    [
      {
        "kind": "string"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "log.error",
    [
      {
        "kind": "string"
      }
    ],
    {
      "kind": "void"
    }
  ],
  [
    "net.request",
    [
      {
        "kind": "string"
      }
    ],
    {
      "fields": [
        [
          "status",
          {
            "kind": "number"
          }
        ],
        [
          "body",
          {
            "kind": "bytes"
          }
        ]
      ],
      "kind": "object"
    }
  ],
  [
    "utility.invoke",
    [
      {
        "kind": "string"
      },
      {
        "kind": "bigint"
      },
      {
        "kind": "optional",
        "value": {
          "kind": "bytes"
        }
      }
    ],
    {
      "kind": "bytes"
    }
  ],
  [
    "utility.cancel",
    [
      {
        "kind": "string"
      },
      {
        "kind": "bigint"
      }
    ],
    {
      "kind": "void"
    }
  ],
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
], [
]);
