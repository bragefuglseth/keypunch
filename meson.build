project('keypunch', 'rust',
          version: '6.3',
    meson_version: '>= 0.62.0',
  default_options: [ 'warning_level=2', 'werror=false', ],
)

i18n = import('i18n')
gnome = import('gnome')

# APP ID
tld = 'no'
domain = 'bragefuglseth'
app_name = 'Keypunch'

base_id = '@0@.@1@.@2@'.format(tld, domain, app_name)
base_path = '/@0@/@1@/@2@/'.format(tld, domain, app_name)

app_id = base_id
if get_option('buildtype') != 'release'
  app_id += '.Devel'
endif

conf = configuration_data()
conf.set('APP_ID', app_id)
conf.set('BASE_ID', base_id)
conf.set('BASE_PATH', base_path)
conf.set('VERSION', meson.project_version())
conf.set('GETTEXT_PACKAGE', 'keypunch')
conf.set('LOCALEDIR', get_option('prefix') / get_option('localedir'))
conf.set('PKGDATADIR', get_option('prefix') / get_option('datadir') / meson.project_name())

subdir('po')
subdir('data')
subdir('src')

gnome.post_install(
     glib_compile_schemas: true,
    gtk_update_icon_cache: true,
  update_desktop_database: true,
)
