# Recettes exécutables des exemples J7

F-CFG-008 relève de RS-PLATFORM-J7. Le catalogue de l’aide fournit à la fois
les consignes traduites et les octets des projets au menu de l’atelier.
[Exemples et résultats attendus](../agnostique/EXEMPLES_J7.md)

Les essais de référence chargent les mêmes fichiers que le menu. Les recettes
peuvent configurer des actions supplémentaires, notamment les règles ordonnées
du pare-feu et les extrémités des connexions de test NAT.

| Famille | Commande |
| --- | --- |
| Connexion directe | `cargo test -p sns-engine --test lan --locked --offline direct_example_resolves_arp_and_replies_over_its_only_cable` |
| LAN | `cargo test -p sns-engine --test lan --locked --offline r_lan_arp_learning_unicast_cache_and_purge` |
| Routes statiques | `cargo test -p sns-engine --test routing --locked --offline three_lans_forward_original_addresses_and_ttl_and_return_real_icmp` |
| RIP | `cargo test -p sns-engine --test routing --locked --offline rip_three_routers_shortest_path_poisoning_and_expiration` |
| DHCP | `cargo test -p sns-engine --test services --locked --offline r_dhcp_two_clients_reservation_effective_settings_and_document` |
| DNS | `cargo test -p sns-engine --test services --locked --offline dns_ttl_cache_is_per_host_and_delete_survives_reload` |
| Web | `cargo test -p sns-engine --test services --locked --offline r_web_offline_dns_http_images_forms_virtual_hosts_and_isolation` |
| Courrier | `cargo test -p sns-engine --test mail --locked --offline smtp_pop3_local_delivery_and_durable_client_folders` |
| NAT | `cargo test -p sns-engine --test private --locked --offline nat_tcp_same_ports_restore_to_correct_hosts_and_keep_original_trace` |
| Pare-feu | `cargo test -p sns-engine --test private --locked --offline firewall_order_policy_icmp_and_udp_reverse_matching` |
| ARP | `cargo test -p sns-engine --test routing --locked --offline arp_exercise_redirects_a_frame_to_the_claiming_host_only_in_the_simulation` |
| Modem | `cargo test -p sns-bridge --test bridge --locked --offline two_real_instances_exchange_ping_record_external_inputs_and_disconnect` |
| P2P | `cargo test -p sns-engine --test p2p --locked --offline cyclic_overlay_discovers_searches_downloads_exact_bytes_and_resets` |

La suite `cargo xtask ci` exécute ces recettes. Les tests de `sns-settings`
valident les 14 projets embarqués et leurs entrées d’aide dans les six langues.
Le test du menu charge les quatre nouveaux projets dans l’atelier et vérifie
son état initial avant activation de la simulation.
