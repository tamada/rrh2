use clap::{Parser, ValueEnum};
use std::{fmt::Display, path::PathBuf, process::ExitStatus};

pub type Result<T> = std::result::Result<T, RrhError>;

#[derive(Debug)]
pub enum RrhError {
    GroupNotFound(String),
    RepositoryNotFound(String),
    RelationNotFound(String, String),
    RepositoryExists(String),
    GroupExists(String),
    RepositoryPathNotFound(PathBuf),
    CliOptsInvalid(String, String),
    Arrays(Vec<RrhError>),
    IO(std::io::Error),
    Json(serde_json::Error),
    Git(git2::Error),
    Arguments(String),
    Fatal(String),
    ExternalCommand(ExitStatus, String),
    Unknown,
}

impl Display for RrhError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Parser, Debug)]
#[clap(
    version,
    author,
    about,
    arg_required_else_help = true,
    allow_external_subcommands = true
)]
pub(crate) struct CliOpts {
    #[arg(
        long = "config-file",
        value_name = "FILE",
        help = "Path to the configuration file"
    )]
    pub(crate) config_file: Option<PathBuf>,

    #[arg(short, long, help = "Verbose mode")]
    pub(crate) verbose: bool,

    #[clap(subcommand)]
    pub(crate) command: Option<RrhCommand>,

    #[arg(index = 1, help = "arguments")]
    pub(crate) args: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) enum RrhCommand {
    #[command(
        name = "add",
        about = "Add repositories to the rrh database (alias of \"repository add\")"
    )]
    Add(AddOpts),

    #[command(
        name = "alias",
        about = "Manage alias (different names of the commands)",
        long_about = "Manage alias (different names of the commands)
    list (no arguments give the registered aliases)
	    alias
    register (\"--\" means skip option parsing after that)
        alias grlist -- repository list --entry group,id
    update
        alias grlist --update -- repository list --entry id
    remove
        alias --remove grlist
    execute
        type the registered alias name instead of rrh sub command"
    )]
    Alias(AliasOpts),

    #[command(
        name = "clone",
        about = "Run \"git clone\" and register its repository to a group"
    )]
    Clone(CloneOpts),

    #[command(name = "find", about = "Find the repositories by the given keyword")]
    Find(FindOpts),

    #[command(
        name = "exec",
        about = "Execute the given command on the specified repositories"
    )]
    Exec(ExecOpts),

    #[command(name = "export", about = "Export the rrh database")]
    Export(ExportOpts),

    #[command(name = "group", about = "Manage the groups for the rrh database")]
    Group(GroupOpts),

    #[command(
        name = "list",
        about = "List the repositories. (alias of \"repository list\")"
    )]
    List(RepositoryListOpts),

    #[command(
        name = "init",
        about = "Generate the shell functions for initializing rrh"
    )]
    Init(InitOpts),

    #[command(
        name = "open",
        about = "Open the folder or web page of the given repositories"
    )]
    Open(OpenOpts),

    #[command(
        name = "prune",
        about = "Prune the database (remove the non-existing repositories)"
    )]
    Prune,

    #[command(
        name = "repository",
        about = "Manage the repositories for the rrh database"
    )]
    Repository(RepositoryOpts),

    #[command(name = "recent", about = "List the recent updated repositories")]
    Recent(RecentOpts),
    // #[command(name = "rm")]
    // Remove(RemoveOpts),
}

#[derive(Parser, Debug)]
pub(crate) struct AddOpts {
    #[clap(flatten)]
    pub(crate) repo: RepositoryOption,

    #[arg(
        help = "repository paths",
        value_name = "REPOSITORIES",
        required = true
    )]
    pub(crate) paths: Vec<PathBuf>,
}

#[derive(Parser, Debug)]
pub(crate) struct RepositoryOption {
    #[arg(
        short = 'r',
        long = "repository-id",
        value_name = "ID",
        help = "Specify the repository ID"
    )]
    pub(crate) repository_id: Option<String>,

    #[clap(flatten, help = "register repositories to the groups.")]
    pub(crate) groups: GroupSpecifier,

    #[arg(
        short = 'd',
        long,
        value_name = "DESCRIPTION",
        help = "Specify the description of the repository"
    )]
    pub(crate) description: Option<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct GroupSpecifier {
    #[arg(short, long, value_name = "GROUPS")]
    pub(crate) group_names: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct RepositorySpecifier {
    #[arg(short, long, value_name = "REPO_IDS")]
    pub(crate) repository_ids: Vec<String>,
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct AliasOpts {
    #[arg(short, long, help = "register repositories to the group.")]
    pub(crate) update: bool,

    #[arg(short, long, help = "register repositories to the group.")]
    pub(crate) remove: bool,

    #[arg(help = "alias name", value_name = "ALIAS_NAME", index = 1)]
    pub(crate) alias: Option<String>,

    #[arg(
        help = "command and its arguments for the alias",
        value_name = "COMMANDS",
        index = 2
    )]
    pub(crate) arguments: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct CloneOpts {
    #[arg(
        short = 'o',
        long = "output",
        value_name = "OUTPUT_DIR",
        help = "output directory for clone",
        default_value = "."
    )]
    pub(crate) dest_dir: PathBuf,

    #[clap(flatten)]
    pub(crate) repo: RepositoryOption,

    #[arg(help = "repository URL", value_name = "REPO_URL")]
    pub(crate) repo_url: String,
}

#[derive(Parser, Debug)]
pub(crate) struct ExecOpts {
    #[clap(
        flatten,
        help = "specify the groups for executing the commands on the corresponding repositories"
    )]
    pub(crate) groups: GroupSpecifier,

    #[clap(flatten, help = "specify the repositories for executing the commands")]
    pub(crate) repositories: RepositorySpecifier,

    #[clap(long = "no-header", help = "do not show the header")]
    pub(crate) no_header: bool,

    #[arg(
        help = "command and its arguments for the alias",
        value_name = "COMMANDS"
    )]
    pub(crate) arguments: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct ExportOpts {
    #[arg(
        short,
        long,
        help = "specify the destination file. \"-\" means stdout",
        value_name = "FILE",
        default_value = "-"
    )]
    pub(crate) dest: String,

    #[arg(short, long, help = "overwrite mode")]
    pub(crate) overwrite: bool,

    #[arg(
        long = "no-replace-home",
        help = "does not replace the home directory to the word \"${HOME}\""
    )]
    pub(crate) no_replace_home: bool,

    #[arg(short, long, help = "indent the resultant json file")]
    pub(crate) indent: bool,
}

#[derive(Parser, Debug)]
pub(crate) struct FindOpts {
    #[arg(
        long,
        help = "This flag turns the keywords into the AND condition. (default is OR)"
    )]
    pub(crate) and: bool,

    #[arg(
        help = "keywords for finding the repositories",
        value_name = "KEYWORDS",
        required = true
    )]
    pub(crate) keywords: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct GroupOpts {
    #[clap(subcommand)]
    pub(crate) subcmd: GroupSubCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum GroupSubCommand {
    #[command(name = "add", about = "Add the groups to the rrh database")]
    Add(GroupAddOpts),

    #[command(name = "info", about = "Show the information of the groups")]
    Info(GroupInfoOpts),

    #[command(name = "list", about = "List the groups")]
    List(GroupListOpts),

    #[command(name = "remove", about = "Remove the groups from the rrh database")]
    Remove(GroupRemoveOpts),

    #[command(name = "update", about = "Update the groups in the rrh database")]
    Update(GroupUpdateOpts),
}

#[derive(Parser, Debug)]
pub(crate) struct GroupAddOpts {
    #[arg(short, long, help = "specify the abbrev flag")]
    pub(crate) abbrev: bool,

    #[arg(short, long, help = "specify the note of group", value_name = "NOTE")]
    pub(crate) note: Option<String>,

    #[arg(
        help = "specify the group names",
        required = true,
        value_name = "GROUPS"
    )]
    pub(crate) names: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct GroupInfoOpts {
    #[arg(
        help = "specify the group names for showing the information",
        value_name = "GROUPS",
        required = true
    )]
    pub(crate) names: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct GroupListOpts {
    #[arg(short, long, help = "specify the entries", use_value_delimiter = true)]
    pub(crate) entries: Vec<GroupEntry>,
}

#[derive(Parser, Debug, ValueEnum, Clone, PartialEq, Eq)]
pub(crate) enum GroupEntry {
    Name,
    Abbrev,
    Note,
    Count,
}

#[derive(Parser, Debug)]
pub(crate) struct GroupRemoveOpts {
    #[arg(short, long, help = "force remove the group")]
    pub(crate) force: bool,
}

#[derive(Parser, Debug)]
pub(crate) struct GroupUpdateOpts {
    #[arg(
        short,
        long,
        help = "specify the abbrev flag",
        value_name = "ABBREV_FLAG"
    )]
    pub(crate) abbrev: Option<bool>,

    #[arg(
        short = 'N',
        long,
        help = "specify the note of group",
        value_name = "NOTE"
    )]
    pub(crate) note: Option<String>,

    #[arg(
        short = 'r',
        long = "rename-to",
        help = "specify the new group name",
        value_name = "NEW_NAME"
    )]
    pub(crate) rename_to: Option<String>,

    #[arg(help = "specify the group name", required = true, value_name = "GROUP")]
    pub(crate) name: String,
}

#[derive(Parser, Debug)]
pub(crate) struct InitOpts {
    #[arg(long, help = "not generate the cdrrh function")]
    pub(crate) without_cdrrh: bool,

    #[arg(long, help = "not generate the rrhpeco function")]
    pub(crate) without_rrhpeco: bool,

    #[arg(long, help = "not generate the rrhfzf function")]
    pub(crate) without_rrhfzf: bool,

    #[arg(
        index = 1,
        value_name = "SHELL_NAME",
        help = "specify the target shell",
        required = true
    )]
    pub(crate) shell_name: ShellName,
}

#[derive(Parser, Debug, ValueEnum, Clone, PartialEq, Eq)]
pub(crate) enum ShellName {
    Bash,
    Zsh,
    Fish,
    Elvish,
    Powershell,
}

#[derive(Parser, Debug)]
pub(crate) struct OpenOpts {
    #[clap(short = 't', long = "target", value_name = "TARGET", required = false,
        help = "Open the folder, web page or project page of the given repositories",
        value_enum, default_value_t = OpenTarget::Folder
    )]
    pub(crate) target: OpenTarget,

    #[clap(value_name = "REPOSITORY/GROUP", required = true, index = 1, help = "specify the open target repositories or groups")]
    pub(crate) args: Vec<String>,
}

#[derive(Debug, ValueEnum, PartialEq, Clone)]
pub(crate) enum OpenTarget {
    Folder,
    Webpage,
    Project,
}

#[derive(Parser, Debug)]
pub(crate) struct RepositoryOpts {
    #[clap(subcommand)]
    pub(crate) subcmd: RepositorySubCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum RepositorySubCommand {
    #[command(name = "add")]
    Add(AddOpts),
    #[command(name = "info")]
    Info(RepositoryInfoOpts),
    #[command(name = "list")]
    List(RepositoryListOpts),
    #[command(name = "remove")]
    Remove(RepositoryRemoveOpts),
    #[command(name = "update")]
    Update(RepositoryUpdateOpts),
}

#[derive(Parser, Debug)]
pub(crate) struct RepositoryPrintingOpts {
    #[arg(long = "no-header", help = "do not show the header", default_value_t = false)]
    pub(crate) no_headers: bool,

    #[arg(
        short,
        long,
        help = "specify the entries",
        value_name = "ENTRIES",
        rename_all = "kebab-case",
        use_value_delimiter = true,
    )]
    pub(crate) entries: Vec<RepositoryEntry>,

    #[arg(short, long, help = "specify the result format", value_name = "FORMAT")]
    pub(crate) format: Option<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct RepositoryInfoOpts {
    #[clap(flatten)]
    pub(crate) printOpts: RepositoryPrintingOpts,

    #[arg(
        help = "specify the ids for the target repositories",
        required = true,
        value_name = "REPOSITORY_ID"
    )]
    pub(crate) ids: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct RepositoryListOpts {
    #[clap(flatten)]
    pub(crate) printOpts: RepositoryPrintingOpts,

    #[arg(
        help = "specify the group names for listing the repositories",
        value_name = "GROUPS"
    )]
    pub(crate) groups: Vec<String>,
}

#[derive(Parser, Debug, ValueEnum, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub(crate) enum RepositoryEntry {
    Id,
    Path,
    Groups,
    Description,
    LastAccess,
    All,
}

#[derive(Parser, Debug)]
pub(crate) struct RepositoryRemoveOpts {
    #[arg(index = 1, help = "specify the ids for the target repositories")]
    pub(crate) ids: Vec<String>,
}

#[derive(Parser, Debug)]
pub(crate) struct RepositoryUpdateOpts {
    #[arg(
        short,
        long,
        help = "specify the new description",
        value_name = "DESCRIPTION"
    )]
    pub(crate) description: Option<String>,

    #[arg(
        short,
        long,
        help = "specify the new repository path",
        value_name = "REPOSITORY_PATH"
    )]
    pub(crate) path: Option<PathBuf>,

    #[arg(short, long, help = "specify the new repository id", value_name = "ID")]
    pub(crate) id: Option<String>,

    #[arg(
        short = 'g',
        long = "groups",
        help = "specify the group names for appending",
        value_name = "GROUPS",
        use_value_delimiter = true
    )]
    pub(crate) groups: Vec<String>,

    #[arg(
        short = 'G',
        long = "new-groups",
        help = "specify the new group names for replacing",
        value_name = "GROUPS",
        use_value_delimiter = true
    )]
    pub(crate) new_groups: Vec<String>,

    #[arg(
        help = "specify the id of the target repository",
        required = true,
        value_name = "REPOSITORY_ID"
    )]
    pub(crate) repository_id: String,
}

#[derive(Parser, Debug)]
pub(crate) struct RecentOpts {
    #[arg(
        short,
        long,
        help = "specify the number of recent repositories",
        value_name = "NUMBER"
    )]
    pub(crate) number: Option<usize>,
}
