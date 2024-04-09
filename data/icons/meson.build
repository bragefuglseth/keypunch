application_id = 'dev.bragefuglseth.Keypunch'

scalable_dir = 'hicolor' / 'scalable' / 'apps'
install_data(
  '@0@.svg'.format(application_id),
  install_dir: get_option('datadir') / 'icons' / scalable_dir
)

symbolic_dir = 'hicolor' / 'symbolic' / 'apps'
install_data(
  '@0@-symbolic.svg'.format(application_id),
  install_dir: get_option('datadir') / 'icons' / symbolic_dir
)
