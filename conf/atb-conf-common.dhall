let type = ./atb-conf-type.dhall

in let Database = {
  sqlite3: type.Sqlite3,
  version: type.Version
}

in let Api = {
  host: Text,
  port: Natural
}

in let Conf = {
  database: Database,
  api: Api
}

in let makeConf
    : Conf -> Conf
    = \(c : Conf) -> {
      database = c.database,
      api = c.api
    }

in  makeConf
