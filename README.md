## At-Boom

At-Boom is system of develop for Mechanical Trading

## Setup

```
cd conf
cp atb-conf.template atb-conf.dhall && vim atb-conf.dhall
cat atb-conf.dhall | dhall-to-yaml-ng > atb-conf.yaml
export PATH_ATB_CONFIG={atb-conf.yaml}
```

## License

This project is licensed under the MIT License.

See [LICENSE](https://github.com/didy-kpn/at-boom/blob/master/LICENSE) for details.
