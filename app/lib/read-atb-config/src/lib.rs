extern crate yaml_rust;

use std::io::prelude::*;

#[allow(dead_code)]
#[derive(Clone)]
pub struct AtbConf {
    yaml: yaml_rust::Yaml
}

impl AtbConf {

    #[allow(dead_code)]
    pub fn load_conf() -> Option<AtbConf> {
        // 環境変数PATH_ATB_CONFIGの値を読み込む
        let result_env_atb_config = std::env::var("PATH_ATB_CONFIG");
        if let Err(_) = result_env_atb_config {
            return None;
        }
        let env_atb_config = result_env_atb_config.unwrap();

        // Yamlファイルを開く
        let file = std::fs::File::open(env_atb_config);
        if let Err(_) = file {
            return None;
        }

        // Yamlファイルを読み込む
        let mut contents = String::new();
        if let Err(_) = file.unwrap().read_to_string(&mut contents) {
            return None
        }

        // 読み込んだYamlファイルをオブジェクトに変換する
        let result_atb_config = yaml_rust::YamlLoader::load_from_str(&contents);
        if let Err(_) = result_atb_config {
            return None
        }

        let atb_config = &result_atb_config.unwrap()[0];
        // println!("{:?}", atb_config);
        Some(AtbConf { yaml: atb_config.clone()})
    }

    #[allow(dead_code)]
    pub fn get_sqlite3_file(&self) -> Option<String> {
        return if let Some(file) = self.yaml["database"]["sqlite3"]["db_file"].as_str() {
            Some(file.to_string())
        } else {
            None
        };
    }

    #[allow(dead_code)]
    pub fn has_api_host(&self) -> bool {
        return self.yaml["api"]["host"].as_str().is_some();
    }

    #[allow(dead_code)]
    pub fn has_api_port(&self) -> bool {
        return self.yaml["api"]["port"].as_i64().is_some();
    }

    #[allow(dead_code)]
    pub fn get_api_host(&self) -> Option<&str> {
        return self.yaml["api"]["host"].as_str();
    }

    #[allow(dead_code)]
    pub fn get_api_port(&self) -> Option<i64> {
        return self.yaml["api"]["port"].as_i64();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        if let Some(atb_conf) = AtbConf::load_conf() {
            println!("{:?}", atb_conf.get_sqlite3_file());
            println!("{:?}", atb_conf.get_api_host());
            println!("{:?}", atb_conf.get_api_port());
        }
        assert_eq!(2 + 2, 4);
    }
}
