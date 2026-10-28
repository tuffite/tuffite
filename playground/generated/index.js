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
    '0x6aa086be721589f9',
    '0xddfa86bcb2114566',
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
    'playground.Aux.RoundTrip'
  ],
  [
    'aux.readState',
    '0x114ab85e38e3986d',
    '0xe2c4c159cf6e38dc',
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
    'playground.Aux.ReadState'
  ],
  [
    'aux.processId',
    '0x1309d795517ce447',
    '0xc4180e34f8ded239',
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
    'playground.Aux.ProcessId'
  ],
  [
    'browserDiagnostics.identity',
    '0x58f4c53e02c8f05d',
    '0x9ad89e2d1099976c',
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
    'playground.BrowserDiagnostics.Identity'
  ],
  [
    'browserDiagnostics.increment',
    '0x902e376bc585b70d',
    '0xa8f9759d8049db8e',
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
    'playground.BrowserDiagnostics.Increment'
  ],
  [
    'commands.doSomething',
    '0x83e837af20d116fb',
    '0x65a05c3ac7ee7b1e',
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
    'playground.Commands.DoSomething'
  ],
  [
    'commands.checkUtilitySession',
    '0xc61aa21d21833df3',
    '0xff327e169194e4d0',
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
    'playground.Commands.CheckUtilitySession'
  ],
  [
    'commands.checkJavascriptExtension',
    '0x06527c3a9967b4a2',
    '0x43d897277b682c0f',
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
    'playground.Commands.CheckJavascriptExtension'
  ],
  [
    'commands.echoMojom',
    '0x1595597800d6d178',
    '0x2ffa4399d9863635',
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
    'playground.Commands.EchoMojom'
  ],
  [
    'diagnostics.processId',
    '0xd81ad5fc08d614b0',
    '0xdcbe258aaed26b7d',
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
    'playground.Diagnostics.ProcessId'
  ],
  [
    'echo.roundTrip',
    '0x58af025b7b66ca5e',
    '0xddfa86bcb2114566',
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
    'playground.Echo.RoundTrip'
  ],
  [
    'echo.restricted',
    '0x9329762a25f6f9f9',
    '0x443926037b7cf97a',
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
    'playground.Echo.Restricted'
  ],
  [
    'profiles.list',
    '0x6fef76834bd415be',
    '0x9ad89e2d1099976c',
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
    'playground.Profiles.List'
  ],
  [
    'profiles.create',
    '0xb691e42765a2ab02',
    '0x24abbf5509a26ced',
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
    'playground.Profiles.Create'
  ],
  [
    'profiles.prepare',
    '0x37cc2b2ea0c4226e',
    '0x21f48f427f6833a5',
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
    'playground.Profiles.Prepare'
  ],
  [
    'profiles.attach',
    '0x93d6a32b82429b9b',
    '0x8e17797eb4ceef74',
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
    'playground.Profiles.Attach'
  ],
  [
    'profiles.releaseView',
    '0x635cd6d2c39d37e7',
    '0x4324574ebaf451d5',
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
    'playground.Profiles.ReleaseView'
  ],
  [
    'profiles.close',
    '0x0c4cb572c1aea249',
    '0x54046f993e6dd968',
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
    'playground.Profiles.Close'
  ],
  [
    'profiles.applicationState',
    '0x216bd68c91e2d7b1',
    '0x35935a08b94ad2b6',
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
    'playground.Profiles.ApplicationState'
  ],
  [
    'profiles.createWindow',
    '0x71e729b5b1afad66',
    '0x3b5dacf6c469b9a8',
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
    'playground.Profiles.CreateWindow'
  ],
  [
    'profiles.closeWindow',
    '0x9949081edfe9825b',
    '0xf2991096defc8c85',
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
    'playground.Profiles.CloseWindow'
  ]
];
import { createMojomCommandApi, invokeBinary } from '/tuffite.js';
export function createPlaygroundApi(transport = invokeBinary) {
  return createMojomCommandApi(mojomMethods, 'playground', transport);
}
export const { aux, browserDiagnostics, commands, diagnostics, echo, profiles } = createPlaygroundApi();
