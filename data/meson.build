desktop_conf = configuration_data()
desktop_conf.set('icon', app_id)

desktop_file = i18n.merge_file(
        input: configure_file(
	    input: '@0@.desktop.in.in'.format(base_id),
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

run_command('cp', '@0@.metainfo.xml.in.in'.format(base_id), '@0@.metainfo.xml.in'.format(base_id), check: true)

# Sed is used as a poor man's `configure_file` here because we need access to it in the src dir

if build_machine.system() in ['freebsd', 'netbsd', 'openbsd', 'dragonfly']
  meson.override_find_program('sed', find_program('gsed'))
endif

sed = find_program('gsed', 'sed', required: true) # We should probably still prefer gsed if it exists
run_command(sed, '-i', 's/@app-id@/' + app_id + '/g', base_id + '.metainfo.xml.in', check: true)

appstream_file = i18n.merge_file(
        input: '@0@.metainfo.xml.in'.format(base_id),
       output: '@0@.metainfo.xml'.format(base_id),
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
    '--template=@0@.metainfo.xml.in'.format(base_id),
    '--output-file=' + 'resources' / '@0@.metainfo.xml'.format(base_id),
    check: true,
)

configure_file(
  input: '@0@.gschema.xml.in'.format(base_id),
  output: '@0@.gschema.xml'.format(app_id),
  configuration: conf,
  install: true,
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas',
)

compile_schemas = find_program('glib-compile-schemas', required: false, disabler: true)
test('Validate schema file',
     compile_schemas,
     args: ['--strict', '--dry-run', meson.current_source_dir()])

subdir('artwork' / 'icon')
