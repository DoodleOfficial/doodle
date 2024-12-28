#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data_path = Path::new("../../data/test.zim");
        if !data_path.exists() {
            return;
        }

        let zim = ZimFile::open(data_path).unwrap();

        assert_eq!(zim.header.magic, 72_173_914);
        assert_eq!(zim.header.major_version, 5);
        assert_eq!(zim.header.minor_version, 0);

        let first_article = zim
            .dir_entries()
            .find(|x| match x {
                Ok(DirEntry::Content { namespace, .. }) => *namespace == 'A',
                _ => false,
            })
            .unwrap()
            .unwrap();

        let url = match first_article {
            DirEntry::Content { url, .. } => url,
            _ => panic!(),
        };

        assert_eq!(url, "African_Americans");
        assert_eq!(zim.dir_entries().count(), 8477);
    }
}
