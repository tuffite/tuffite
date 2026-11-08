// Generated from Mojom v0. Do not edit.
export const mojomTypes = {
  'AuxProcessIdRequest': {
    'fields': [],
    'size': 8
  },
  'AuxProcessIdResponse': {
    'fields': [[
      'result',
      8,
      0,
      'u32'
    ]],
    'size': 16
  },
  'AuxReadStateRequest': {
    'fields': [],
    'size': 8
  },
  'AuxReadStateResponse': {
    'fields': [[
      'result',
      8,
      0,
      'u64'
    ]],
    'size': 16
  },
  'AuxRoundTripRequest': {
    'fields': [[
      'value',
      8,
      0,
      {
        'fields': [
          [
            'small',
            8,
            0,
            'u8'
          ],
          [
            'large',
            16,
            0,
            'u64'
          ],
          [
            'enabled',
            9,
            0,
            'bool'
          ],
          [
            'visible',
            9,
            1,
            'bool'
          ],
          [
            'medium',
            10,
            0,
            'u16'
          ],
          [
            'signedValue',
            12,
            0,
            'i32'
          ],
          [
            'fraction',
            24,
            0,
            'f64'
          ],
          [
            'nested',
            32,
            0,
            {
              'fields': [[
                'label',
                8,
                0,
                'string'
              ], [
                'bytes',
                16,
                0,
                'bytes'
              ]],
              'size': 24
            }
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'AuxRoundTripResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'small',
            8,
            0,
            'u8'
          ],
          [
            'large',
            16,
            0,
            'u64'
          ],
          [
            'enabled',
            9,
            0,
            'bool'
          ],
          [
            'visible',
            9,
            1,
            'bool'
          ],
          [
            'medium',
            10,
            0,
            'u16'
          ],
          [
            'signedValue',
            12,
            0,
            'i32'
          ],
          [
            'fraction',
            24,
            0,
            'f64'
          ],
          [
            'nested',
            32,
            0,
            {
              'fields': [[
                'label',
                8,
                0,
                'string'
              ], [
                'bytes',
                16,
                0,
                'bytes'
              ]],
              'size': 24
            }
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'BrowserDiagnosticsIdentityRequest': {
    'fields': [],
    'size': 8
  },
  'BrowserDiagnosticsIdentityResponse': {
    'fields': [[
      'result',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'BrowserDiagnosticsIncrementRequest': {
    'fields': [],
    'size': 8
  },
  'BrowserDiagnosticsIncrementResponse': {
    'fields': [[
      'result',
      8,
      0,
      'u32'
    ]],
    'size': 16
  },
  'CommandsCheckJavascriptExtensionRequest': {
    'fields': [],
    'size': 8
  },
  'CommandsCheckJavascriptExtensionResponse': {
    'fields': [[
      'result',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'CommandsCheckUtilitySessionRequest': {
    'fields': [],
    'size': 8
  },
  'CommandsCheckUtilitySessionResponse': {
    'fields': [[
      'result',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'CommandsDoSomethingRequest': {
    'fields': [[
      'message',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'CommandsDoSomethingResponse': {
    'fields': [[
      'result',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'CommandsEchoMojomRequest': {
    'fields': [[
      'value',
      8,
      0,
      {
        'fields': [
          [
            'small',
            8,
            0,
            'u8'
          ],
          [
            'large',
            16,
            0,
            'u64'
          ],
          [
            'enabled',
            9,
            0,
            'bool'
          ],
          [
            'visible',
            9,
            1,
            'bool'
          ],
          [
            'medium',
            10,
            0,
            'u16'
          ],
          [
            'signedValue',
            12,
            0,
            'i32'
          ],
          [
            'fraction',
            24,
            0,
            'f64'
          ],
          [
            'nested',
            32,
            0,
            {
              'fields': [[
                'label',
                8,
                0,
                'string'
              ], [
                'bytes',
                16,
                0,
                'bytes'
              ]],
              'size': 24
            }
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'CommandsEchoMojomResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'small',
            8,
            0,
            'u8'
          ],
          [
            'large',
            16,
            0,
            'u64'
          ],
          [
            'enabled',
            9,
            0,
            'bool'
          ],
          [
            'visible',
            9,
            1,
            'bool'
          ],
          [
            'medium',
            10,
            0,
            'u16'
          ],
          [
            'signedValue',
            12,
            0,
            'i32'
          ],
          [
            'fraction',
            24,
            0,
            'f64'
          ],
          [
            'nested',
            32,
            0,
            {
              'fields': [[
                'label',
                8,
                0,
                'string'
              ], [
                'bytes',
                16,
                0,
                'bytes'
              ]],
              'size': 24
            }
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'DiagnosticsProcessIdRequest': {
    'fields': [],
    'size': 8
  },
  'DiagnosticsProcessIdResponse': {
    'fields': [[
      'result',
      8,
      0,
      'u32'
    ]],
    'size': 16
  },
  'EchoRestrictedRequest': {
    'fields': [[
      'value',
      8,
      0,
      {
        'fields': [
          [
            'small',
            8,
            0,
            'u8'
          ],
          [
            'large',
            16,
            0,
            'u64'
          ],
          [
            'enabled',
            9,
            0,
            'bool'
          ],
          [
            'visible',
            9,
            1,
            'bool'
          ],
          [
            'medium',
            10,
            0,
            'u16'
          ],
          [
            'signedValue',
            12,
            0,
            'i32'
          ],
          [
            'fraction',
            24,
            0,
            'f64'
          ],
          [
            'nested',
            32,
            0,
            {
              'fields': [[
                'label',
                8,
                0,
                'string'
              ], [
                'bytes',
                16,
                0,
                'bytes'
              ]],
              'size': 24
            }
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'EchoRestrictedResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'small',
            8,
            0,
            'u8'
          ],
          [
            'large',
            16,
            0,
            'u64'
          ],
          [
            'enabled',
            9,
            0,
            'bool'
          ],
          [
            'visible',
            9,
            1,
            'bool'
          ],
          [
            'medium',
            10,
            0,
            'u16'
          ],
          [
            'signedValue',
            12,
            0,
            'i32'
          ],
          [
            'fraction',
            24,
            0,
            'f64'
          ],
          [
            'nested',
            32,
            0,
            {
              'fields': [[
                'label',
                8,
                0,
                'string'
              ], [
                'bytes',
                16,
                0,
                'bytes'
              ]],
              'size': 24
            }
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'EchoRoundTripRequest': {
    'fields': [[
      'value',
      8,
      0,
      {
        'fields': [
          [
            'small',
            8,
            0,
            'u8'
          ],
          [
            'large',
            16,
            0,
            'u64'
          ],
          [
            'enabled',
            9,
            0,
            'bool'
          ],
          [
            'visible',
            9,
            1,
            'bool'
          ],
          [
            'medium',
            10,
            0,
            'u16'
          ],
          [
            'signedValue',
            12,
            0,
            'i32'
          ],
          [
            'fraction',
            24,
            0,
            'f64'
          ],
          [
            'nested',
            32,
            0,
            {
              'fields': [[
                'label',
                8,
                0,
                'string'
              ], [
                'bytes',
                16,
                0,
                'bytes'
              ]],
              'size': 24
            }
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'EchoRoundTripResponse': {
    'fields': [[
      'result',
      8,
      0,
      {
        'fields': [
          [
            'small',
            8,
            0,
            'u8'
          ],
          [
            'large',
            16,
            0,
            'u64'
          ],
          [
            'enabled',
            9,
            0,
            'bool'
          ],
          [
            'visible',
            9,
            1,
            'bool'
          ],
          [
            'medium',
            10,
            0,
            'u16'
          ],
          [
            'signedValue',
            12,
            0,
            'i32'
          ],
          [
            'fraction',
            24,
            0,
            'f64'
          ],
          [
            'nested',
            32,
            0,
            {
              'fields': [[
                'label',
                8,
                0,
                'string'
              ], [
                'bytes',
                16,
                0,
                'bytes'
              ]],
              'size': 24
            }
          ]
        ],
        'size': 40
      }
    ]],
    'size': 16
  },
  'MojomNested': {
    'fields': [[
      'label',
      8,
      0,
      'string'
    ], [
      'bytes',
      16,
      0,
      'bytes'
    ]],
    'size': 24
  },
  'MojomSample': {
    'fields': [
      [
        'small',
        8,
        0,
        'u8'
      ],
      [
        'large',
        16,
        0,
        'u64'
      ],
      [
        'enabled',
        9,
        0,
        'bool'
      ],
      [
        'visible',
        9,
        1,
        'bool'
      ],
      [
        'medium',
        10,
        0,
        'u16'
      ],
      [
        'signedValue',
        12,
        0,
        'i32'
      ],
      [
        'fraction',
        24,
        0,
        'f64'
      ],
      [
        'nested',
        32,
        0,
        {
          'fields': [[
            'label',
            8,
            0,
            'string'
          ], [
            'bytes',
            16,
            0,
            'bytes'
          ]],
          'size': 24
        }
      ]
    ],
    'size': 40
  },
  'ProfilesApplicationStateRequest': {
    'fields': [],
    'size': 8
  },
  'ProfilesApplicationStateResponse': {
    'fields': [[
      'result',
      8,
      0,
      'u32'
    ]],
    'size': 16
  },
  'ProfilesAttachRequest': {
    'fields': [[
      'windowId',
      8,
      0,
      'i64'
    ], [
      'view',
      16,
      0,
      'string'
    ]],
    'size': 24
  },
  'ProfilesAttachResponse': {
    'fields': [[
      'result',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'ProfilesCloseRequest': {
    'fields': [[
      'context',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'ProfilesCloseResponse': {
    'fields': [],
    'size': 8
  },
  'ProfilesCloseWindowRequest': {
    'fields': [[
      'windowId',
      8,
      0,
      'i64'
    ]],
    'size': 16
  },
  'ProfilesCloseWindowResponse': {
    'fields': [],
    'size': 8
  },
  'ProfilesCreateRequest': {
    'fields': [[
      'id',
      8,
      0,
      'string'
    ], [
      'name',
      16,
      0,
      'string'
    ]],
    'size': 24
  },
  'ProfilesCreateResponse': {
    'fields': [],
    'size': 8
  },
  'ProfilesCreateWindowRequest': {
    'fields': [],
    'size': 8
  },
  'ProfilesCreateWindowResponse': {
    'fields': [[
      'result',
      8,
      0,
      'i64'
    ]],
    'size': 16
  },
  'ProfilesListRequest': {
    'fields': [],
    'size': 8
  },
  'ProfilesListResponse': {
    'fields': [[
      'result',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'ProfilesPrepareRequest': {
    'fields': [[
      'id',
      8,
      0,
      'string'
    ], [
      'url',
      16,
      0,
      'string'
    ]],
    'size': 24
  },
  'ProfilesPrepareResponse': {
    'fields': [[
      'result',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'ProfilesReleaseViewRequest': {
    'fields': [[
      'view',
      8,
      0,
      'string'
    ]],
    'size': 16
  },
  'ProfilesReleaseViewResponse': {
    'fields': [],
    'size': 8
  }
};
export const mojomMethods = [
  [
    'aux.roundTrip',
    0,
    {
      'fields': [[
        'value',
        8,
        0,
        {
          'fields': [
            [
              'small',
              8,
              0,
              'u8'
            ],
            [
              'large',
              16,
              0,
              'u64'
            ],
            [
              'enabled',
              9,
              0,
              'bool'
            ],
            [
              'visible',
              9,
              1,
              'bool'
            ],
            [
              'medium',
              10,
              0,
              'u16'
            ],
            [
              'signedValue',
              12,
              0,
              'i32'
            ],
            [
              'fraction',
              24,
              0,
              'f64'
            ],
            [
              'nested',
              32,
              0,
              {
                'fields': [[
                  'label',
                  8,
                  0,
                  'string'
                ], [
                  'bytes',
                  16,
                  0,
                  'bytes'
                ]],
                'size': 24
              }
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'small',
              8,
              0,
              'u8'
            ],
            [
              'large',
              16,
              0,
              'u64'
            ],
            [
              'enabled',
              9,
              0,
              'bool'
            ],
            [
              'visible',
              9,
              1,
              'bool'
            ],
            [
              'medium',
              10,
              0,
              'u16'
            ],
            [
              'signedValue',
              12,
              0,
              'i32'
            ],
            [
              'fraction',
              24,
              0,
              'f64'
            ],
            [
              'nested',
              32,
              0,
              {
                'fields': [[
                  'label',
                  8,
                  0,
                  'string'
                ], [
                  'bytes',
                  16,
                  0,
                  'bytes'
                ]],
                'size': 24
              }
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    'playground.wire.Aux'
  ],
  [
    'aux.readState',
    1,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'u64'
      ]],
      'size': 16
    },
    'playground.wire.Aux'
  ],
  [
    'aux.processId',
    2,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'u32'
      ]],
      'size': 16
    },
    'playground.wire.Aux'
  ],
  [
    'browserDiagnostics.identity',
    0,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    'playground.wire.BrowserDiagnostics'
  ],
  [
    'browserDiagnostics.increment',
    1,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'u32'
      ]],
      'size': 16
    },
    'playground.wire.BrowserDiagnostics'
  ],
  [
    'commands.doSomething',
    0,
    {
      'fields': [[
        'message',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    'playground.wire.Commands'
  ],
  [
    'commands.checkUtilitySession',
    1,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    'playground.wire.Commands'
  ],
  [
    'commands.checkJavascriptExtension',
    2,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    'playground.wire.Commands'
  ],
  [
    'commands.echoMojom',
    3,
    {
      'fields': [[
        'value',
        8,
        0,
        {
          'fields': [
            [
              'small',
              8,
              0,
              'u8'
            ],
            [
              'large',
              16,
              0,
              'u64'
            ],
            [
              'enabled',
              9,
              0,
              'bool'
            ],
            [
              'visible',
              9,
              1,
              'bool'
            ],
            [
              'medium',
              10,
              0,
              'u16'
            ],
            [
              'signedValue',
              12,
              0,
              'i32'
            ],
            [
              'fraction',
              24,
              0,
              'f64'
            ],
            [
              'nested',
              32,
              0,
              {
                'fields': [[
                  'label',
                  8,
                  0,
                  'string'
                ], [
                  'bytes',
                  16,
                  0,
                  'bytes'
                ]],
                'size': 24
              }
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'small',
              8,
              0,
              'u8'
            ],
            [
              'large',
              16,
              0,
              'u64'
            ],
            [
              'enabled',
              9,
              0,
              'bool'
            ],
            [
              'visible',
              9,
              1,
              'bool'
            ],
            [
              'medium',
              10,
              0,
              'u16'
            ],
            [
              'signedValue',
              12,
              0,
              'i32'
            ],
            [
              'fraction',
              24,
              0,
              'f64'
            ],
            [
              'nested',
              32,
              0,
              {
                'fields': [[
                  'label',
                  8,
                  0,
                  'string'
                ], [
                  'bytes',
                  16,
                  0,
                  'bytes'
                ]],
                'size': 24
              }
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    'playground.wire.Commands'
  ],
  [
    'diagnostics.processId',
    0,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'u32'
      ]],
      'size': 16
    },
    'playground.wire.Diagnostics'
  ],
  [
    'echo.roundTrip',
    0,
    {
      'fields': [[
        'value',
        8,
        0,
        {
          'fields': [
            [
              'small',
              8,
              0,
              'u8'
            ],
            [
              'large',
              16,
              0,
              'u64'
            ],
            [
              'enabled',
              9,
              0,
              'bool'
            ],
            [
              'visible',
              9,
              1,
              'bool'
            ],
            [
              'medium',
              10,
              0,
              'u16'
            ],
            [
              'signedValue',
              12,
              0,
              'i32'
            ],
            [
              'fraction',
              24,
              0,
              'f64'
            ],
            [
              'nested',
              32,
              0,
              {
                'fields': [[
                  'label',
                  8,
                  0,
                  'string'
                ], [
                  'bytes',
                  16,
                  0,
                  'bytes'
                ]],
                'size': 24
              }
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'small',
              8,
              0,
              'u8'
            ],
            [
              'large',
              16,
              0,
              'u64'
            ],
            [
              'enabled',
              9,
              0,
              'bool'
            ],
            [
              'visible',
              9,
              1,
              'bool'
            ],
            [
              'medium',
              10,
              0,
              'u16'
            ],
            [
              'signedValue',
              12,
              0,
              'i32'
            ],
            [
              'fraction',
              24,
              0,
              'f64'
            ],
            [
              'nested',
              32,
              0,
              {
                'fields': [[
                  'label',
                  8,
                  0,
                  'string'
                ], [
                  'bytes',
                  16,
                  0,
                  'bytes'
                ]],
                'size': 24
              }
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    'playground.wire.Echo'
  ],
  [
    'echo.restricted',
    1,
    {
      'fields': [[
        'value',
        8,
        0,
        {
          'fields': [
            [
              'small',
              8,
              0,
              'u8'
            ],
            [
              'large',
              16,
              0,
              'u64'
            ],
            [
              'enabled',
              9,
              0,
              'bool'
            ],
            [
              'visible',
              9,
              1,
              'bool'
            ],
            [
              'medium',
              10,
              0,
              'u16'
            ],
            [
              'signedValue',
              12,
              0,
              'i32'
            ],
            [
              'fraction',
              24,
              0,
              'f64'
            ],
            [
              'nested',
              32,
              0,
              {
                'fields': [[
                  'label',
                  8,
                  0,
                  'string'
                ], [
                  'bytes',
                  16,
                  0,
                  'bytes'
                ]],
                'size': 24
              }
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    {
      'fields': [[
        'result',
        8,
        0,
        {
          'fields': [
            [
              'small',
              8,
              0,
              'u8'
            ],
            [
              'large',
              16,
              0,
              'u64'
            ],
            [
              'enabled',
              9,
              0,
              'bool'
            ],
            [
              'visible',
              9,
              1,
              'bool'
            ],
            [
              'medium',
              10,
              0,
              'u16'
            ],
            [
              'signedValue',
              12,
              0,
              'i32'
            ],
            [
              'fraction',
              24,
              0,
              'f64'
            ],
            [
              'nested',
              32,
              0,
              {
                'fields': [[
                  'label',
                  8,
                  0,
                  'string'
                ], [
                  'bytes',
                  16,
                  0,
                  'bytes'
                ]],
                'size': 24
              }
            ]
          ],
          'size': 40
        }
      ]],
      'size': 16
    },
    'playground.wire.Echo'
  ],
  [
    'profiles.list',
    0,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    'playground.wire.Profiles'
  ],
  [
    'profiles.create',
    1,
    {
      'fields': [[
        'id',
        8,
        0,
        'string'
      ], [
        'name',
        16,
        0,
        'string'
      ]],
      'size': 24
    },
    {
      'fields': [],
      'size': 8
    },
    'playground.wire.Profiles'
  ],
  [
    'profiles.prepare',
    2,
    {
      'fields': [[
        'id',
        8,
        0,
        'string'
      ], [
        'url',
        16,
        0,
        'string'
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    'playground.wire.Profiles'
  ],
  [
    'profiles.attach',
    3,
    {
      'fields': [[
        'windowId',
        8,
        0,
        'i64'
      ], [
        'view',
        16,
        0,
        'string'
      ]],
      'size': 24
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    'playground.wire.Profiles'
  ],
  [
    'profiles.releaseView',
    4,
    {
      'fields': [[
        'view',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'playground.wire.Profiles'
  ],
  [
    'profiles.close',
    5,
    {
      'fields': [[
        'context',
        8,
        0,
        'string'
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'playground.wire.Profiles'
  ],
  [
    'profiles.applicationState',
    6,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'u32'
      ]],
      'size': 16
    },
    'playground.wire.Profiles'
  ],
  [
    'profiles.createWindow',
    7,
    {
      'fields': [],
      'size': 8
    },
    {
      'fields': [[
        'result',
        8,
        0,
        'i64'
      ]],
      'size': 16
    },
    'playground.wire.Profiles'
  ],
  [
    'profiles.closeWindow',
    8,
    {
      'fields': [[
        'windowId',
        8,
        0,
        'i64'
      ]],
      'size': 16
    },
    {
      'fields': [],
      'size': 8
    },
    'playground.wire.Profiles'
  ]
];
import { createMojomCommandApi } from '/tuffite.js';
export function createPlaygroundApi(transport) {
  return createMojomCommandApi(mojomMethods, 'playground', transport);
}
export const { aux, browserDiagnostics, commands, diagnostics, echo, profiles } = createPlaygroundApi();
