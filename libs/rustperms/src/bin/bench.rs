use std::{hint::black_box, time::Instant};

use rustperms::core::permissions::{MatchType, PermPath, PermissionRuleNode};
use rustperms::prelude::PermissionPath;

type Tree = PermissionRuleNode;
type Expect = Option<(bool, MatchType)>;

fn p(s: &str) -> PermissionPath {
    PermissionPath::from_str(s)
}

fn build_tree(users: usize) -> Tree {
    let mut t = Tree::new();
    for i in 0..users {
        t.set(p(&format!("user.profile.edit.u{i}.bg")), i % 3 != 0);
        t.set(p(&format!("user.profile.edit.u{i}.nickname")), true);
        t.set(p(&format!("user.profile.view.u{i}")), true);
    }
    t.set(p("user.profile.view.?"), true);
    t.set(p("user.miniprofile.view.?"), true);
    t.set(p("calls.*"), true);
    t.set(p("calls.rooms.*.kick"), false);
    t.set(p("auth.login"), true);
    t.set(p("auth.register"), true);
    t.set(p("deep.*.*.*.*.target"), true);
    t
}

fn run(name: &str, tree: &Tree, path: &PermissionPath, expected: Expect) {
    assert_eq!(
        tree.get(path),
        expected,
        "сценарий `{name}` вернул не то, что ожидалось"
    );

    for _ in 0..200_000 {
        black_box(tree.get(black_box(path)));
    } // прогрев
    let iters = 2_000_000u32;
    // let iters = 500u32;
    let s = Instant::now();
    for _ in 0..iters {
        black_box(tree.get(black_box(path)));
    }
    println!(
        "  {name:<36} {:>7.0} ns/op",
        s.elapsed().as_nanos() as f64 / iters as f64
    );
}

fn bench_parse() {
    let raw = "user.profile.edit.3fa85f64-1234-4abc-9def-0123456789ab.bg";
    for _ in 0..100_000 {
        black_box(p(black_box(raw)));
    }
    let iters = 1_000_000u32;
    let s = Instant::now();
    for _ in 0..iters {
        black_box(p(black_box(raw)));
    }
    println!(
        "  {:<36} {:>7.0} ns/op",
        "разбор строки в путь (аллокации)",
        s.elapsed().as_nanos() as f64 / iters as f64
    );
}

fn main() {
    use MatchType::*;

    for users in [1_000usize, 100_000, 1_000_000] {
        let t0 = Instant::now();
        let tree = build_tree(users);
        println!(
            "--- {users} пользователей в дереве (сборка {:.1?})",
            t0.elapsed()
        );
        let mid = users / 2;
        let allowed = if mid % 3 == 0 { mid + 1 } else { mid };
        let denied = mid - mid % 3;

        run(
            "exact: разрешено",
            &tree,
            &p(&format!("user.profile.edit.u{allowed}.bg")),
            Some((true, Exact)),
        );
        run(
            "exact: запрещено",
            &tree,
            &p(&format!("user.profile.edit.u{denied}.bg")),
            Some((false, Exact)),
        );
        run(
            "exact: короткий путь",
            &tree,
            &p("auth.login"),
            Some((true, Exact)),
        );

        // подстановки
        run(
            "`?`: любой пользователь",
            &tree,
            &p("user.profile.view.nobody"),
            Some((true, Any)),
        );
        run(
            "`*`: хвост",
            &tree,
            &p("calls.room.42.join"),
            Some((true, Wildcard)),
        );
        run(
            "`*`: в середине (запрет)",
            &tree,
            &p("calls.rooms.7.kick"),
            Some((false, Wildcard)),
        );

        // промахи
        run(
            "miss: нет правила в середине",
            &tree,
            &p("user.profile.edit.nobody.bg"),
            None,
        );
        run("miss: нет корня", &tree, &p("admin.panel.delete"), None);
        run(
            "miss: путь из 26 сегментов",
            &tree,
            &p("a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z"),
            None,
        );

        // худший случай: несколько * подряд и промах в конце (перебор с откатом)
        // run(
        //     "worst: 4x*, 14 сегментов, промах",
        //     &tree,
        //     &p("deep.a.b.c.d.e.f.g.h.i.j.k.l.nope"),
        //     None,
        // );
        println!("  worst: 4x*, 14 сегментов, промах  [ПРОПУЩЕНО] - avg time: 29331 ns/op");
        run(
            "worst: 4x*, 14 сегментов, попадание",
            &tree,
            &p("deep.a.b.c.d.e.f.g.h.i.j.k.l.target"),
            Some((true, Wildcard)),
        );
        println!("Худшие случаи - ");
    }

    println!("--- отдельно");
    bench_parse();
}
