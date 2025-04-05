project('keypunch', 'rust',
          version: '0.6.1',
    meson_version: '>= 0.62.0',
  default_options: [ 'warning_level=2', 'werror=false', ],
)

i18n = import('i18n')
gnome = import('gnome')

app_id = 'dev.bragefuglseth.Keypunch'

if get_option('buildtype') != 'release'
  app_id += '.Devel'
endif

subdir('po')
subdir('data')
subdir('src')

gnome.post_install(
     glib_compile_schemas: true,
    gtk_update_icon_cache: true,
  update_desktop_database: true,
)
