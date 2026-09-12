// Generated from ../../framework/shell_api/remoting.d.ts. Do not edit.

export const entries = [
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
  ]
];
export const constants = [
];
