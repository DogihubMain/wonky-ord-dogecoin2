use {super::*, fee_rate::FeeRate};

#[derive(Debug, Parser)]
pub(crate) struct Preview {
  #[clap(flatten)]
  server: super::server::Server,
  inscriptions: Vec<PathBuf>,
}

struct KillOnDrop(process::Child);

impl Drop for KillOnDrop {
  fn drop(&mut self) {
    self.0.kill().unwrap()
  }
}

impl Preview {
  pub(crate) fn run(self) -> SubcommandResult {
    let tmpdir = TempDir::new()?;

    let rpc_port = TcpListener::bind("0.0.0.0:0")?.local_addr()?.port();

    let dogecoin_data_dir = tmpdir.path().join("dogecoin");

    fs::create_dir(&dogecoin_data_dir)?;

    let _dogecoind = KillOnDrop(
      Command::new("dogecoind")
        .arg({
          let mut arg = OsString::from("-datadir=");
          arg.push(&dogecoin_data_dir);
          arg
        })
        .arg("-regtest")
        .arg("-txindex")
        .arg("-listen=0")
        .arg(format!("-rpcport={rpc_port}"))
        .spawn()
        .context("failed to spawn `dogecoind`")?,
    );

    let options = Options {
      chain_argument: Chain::Regtest,
      dogecoin_data_dir: Some(dogecoin_data_dir),
      data_dir: Some(tmpdir.path().into()),
      rpc_url: Some(format!("localhost:{rpc_port}")),
      index_sats: true,
      ..Options::default()
    };

    for attempt in 0.. {
      if options.dogecoin_rpc_client().is_ok() {
        break;
      }

      if attempt == 100 {
        panic!("Dogecoin Core RPC did not respond");
      }

      thread::sleep(Duration::from_millis(50));
    }

    super::wallet::Wallet::Create(super::wallet::create::Create {
      passphrase: "".into(),
    })
    .run(options.clone())?;

    let rpc_client = options.dogecoin_rpc_client_for_wallet_command(false)?;

    let address =
      rpc_client.get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?;

    rpc_client.generate_to_address(101, &address)?;

    for file in self.inscriptions {
      Arguments {
        options: options.clone(),
        subcommand: Subcommand::Wallet(super::wallet::Wallet::Inscribe(
          super::wallet::inscribe::Inscribe {
            fee_rate: FeeRate::try_from(1.0).unwrap(),
            commit_fee_rate: None,
            file,
            no_backup: true,
            satpoint: None,
            dry_run: false,
            no_limit: false,
            destination: None,
          },
        )),
      }
      .run()?;

      rpc_client.generate_to_address(1, &address)?;
    }

    rpc_client.generate_to_address(1, &address)?;

    Arguments {
      options,
      subcommand: Subcommand::Server(self.server),
    }
    .run()
  }
}
