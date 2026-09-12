gclient_gn_args_from = 'src'

vars = {
  # Update this exact commit when rolling Chromium. The patch tooling reads the
  # same value, so DEPS and patch export have one Chromium baseline.
  'chromium_revision': '03a0332726628875bf4590a9363647c666c6bb95',
  'chromium_git': 'https://chromium.googlesource.com',
  'checkout_chromium': True,
}

deps = {
  'src': {
    'url': Var('chromium_git') + '/chromium/src.git@' +
           Var('chromium_revision'),
    'condition': 'checkout_chromium',
  },
}

hooks = [
  {
    'name': 'apply_tuffite_chromium_patches',
    'pattern': 'src/tuffite',
    'cwd': 'src/tuffite',
    'condition': 'checkout_chromium',
    'action': [
      'python3',
      'x.py',
      'bootstrap',
      '--non-interactive',
    ],
  },
]

recursedeps = [
  'src',
]

