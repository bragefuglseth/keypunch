desktop_file = i18n.merge_file(
        input: 'dev.bragefuglseth.Keypunch.desktop.in',
       output: 'dev.bragefuglseth.Keypunch.desktop',
         type: 'desktop',
       po_dir: '../po',
      install: true,
  install_dir: get_option('datadir') / 'applications'
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

appstream_file = i18n.merge_file(
        input: 'dev.bragefuglseth.Keypunch.metainfo.xml.in',
       output: 'dev.bragefuglseth.Keypunch.metainfo.xml',
       po_dir: '../po',
      install: true,
  install_dir: get_option('datadir') / 'metainfo'
)

appstreamcli = find_program('appstreamcli', required: false, disabler: true)
test('Validate appstream file', appstreamcli,
     args: ['validate', '--no-net', '--explain', appstream_file])

# Setup a translated metainfo file in the resources dir as well
# for use by the about dialog
msgfmt = find_program('msgfmt', required: true)
run_command(msgfmt,
    '--xml',
    '-d' + meson.project_source_root() / 'po',
    '--template=dev.bragefuglseth.Keypunch.metainfo.xml.in',
    '--output-file=' + 'resources' / 'dev.bragefuglseth.Keypunch.metainfo.xml',
    check: true,
)

install_data('dev.bragefuglseth.Keypunch.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas'
)

compile_schemas = find_program('glib-compile-schemas', required: false, disabler: true)
test('Validate schema file',
     compile_schemas,
     args: ['--strict', '--dry-run', meson.current_source_dir()])

subdir('icons')
