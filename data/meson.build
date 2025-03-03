desktop_conf = configuration_data()
desktop_conf.set('icon', app_id)

desktop_file = i18n.merge_file(
        input: configure_file(
	    input: 'dev.bragefuglseth.Keypunch.desktop.in.in',
	    output: '@BASENAME@',
	    configuration: desktop_conf
	  ),
       output: '@0@.desktop'.format(app_id),
         type: 'desktop',
       po_dir: '../po',
      install: true,
  install_dir: get_option('datadir') / 'applications'
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

run_command('cp', 'dev.bragefuglseth.Keypunch.metainfo.xml.in.in', 'dev.bragefuglseth.Keypunch.metainfo.xml.in', check: true)

# Sed is used as a poor man's `configure_file` here because we need access to it in the src dir
sed = find_program('sed', required: true)
run_command(sed, '-i', 's/@app-id@/' + app_id + '/g', 'dev.bragefuglseth.Keypunch.metainfo.xml.in', check: true)

appstream_file = i18n.merge_file(
        input: 'dev.bragefuglseth.Keypunch.metainfo.xml.in',
       output: 'dev.bragefuglseth.Keypunch.metainfo.xml',
       po_dir: '../po',
         type: 'xml',
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

subdir('artwork' / 'icon')
