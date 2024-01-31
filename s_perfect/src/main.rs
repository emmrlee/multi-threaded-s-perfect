// Located in: s_perfect
// Author: Emma Lee
// Date Created: 12/5/2023
/* Purpose: Homework 6
 * Calculate the S perfect and near S perfect numbers up to a given max
 */

use std::env;
use std::thread;
use std::sync::mpsc;

const NUM_THREADS:i32 = 8;
const THREAD_SIZE:i32 = 1000;
// S_DEFICIENT and S_ABUNDANT must be symmetrical
// they represent the range at which a number is 
// considered near s perfect [S_DEFICIENT, S_ABUNDANT]
const S_DEFICIENT:i32 = -7;
const S_ABUNDANT:i32 = 7;
const NEAR_S_RANGE:usize = 14;
const VEC:Vec<i32> = Vec::new(); // a new vector constant to be used for efficient copying

fn main() {
    let args : Vec<String> = env::args().collect();
    let max_size = args[1].trim().parse::<i32>().unwrap();

    let mut s_set:Vec<i32> = vec![];
    s_set.push(1); // initialize the s set

    // store the perfect and near perfect numbers in an array of vectors 
    // to be indexed according to their distance from being s perfect
    let mut s_perf = [VEC; NEAR_S_RANGE + 1];
    
    // initialize thread chunk sizes
    let initial_single_thread_size = 10000;
    let leftovers = max_size - initial_single_thread_size;
    let stop_thread = leftovers/THREAD_SIZE;
    let mut thread_id = 0; // 0 indicates main thread

    // spawn single thread first chunk
    let (mut chunk1_s_set, mut chunk1_s_perfect, _chunk1_id) = 
        s_perfect(2, initial_single_thread_size + 1, thread_id, s_set.clone());
  
    // update s set & s perfect to reflect the additions from the first chunk
    s_set.append(&mut chunk1_s_set);
    for i in 0..s_perf.len() {
        s_perf[i].append(&mut chunk1_s_perfect[i]);
    }
    thread_id += 1;

    // spawn 8 threads using s_set info gathered from the first chunk
    let mut thread_start = initial_single_thread_size + 1;
    let mut thread_end = thread_start + THREAD_SIZE;
    let (transmitter, receiver) = mpsc::channel();
    for _i in 1..=NUM_THREADS {
        let my_transmitter = transmitter.clone(); // create a new transmitter for each thread to avoid moving ownership
        let borrowed_s_set = s_set.clone(); // create a clone of the s_set to avoid moving ownership
        thread::spawn(move || {
            let (my_s_set, my_s_perfect, my_thread_id) = s_perfect(thread_start, thread_end, thread_id, borrowed_s_set);
            let my_additions = (my_s_set, my_s_perfect, my_thread_id);
            my_transmitter.send(my_additions).unwrap();
        });
        
        thread_start = thread_end;
        thread_end += THREAD_SIZE;
        thread_id += 1;
    }

    let mut received_count = 0;
    // for each received additions, add them to s_set and s_perf and spawn a new thread if needed
    for (mut received_s_set, mut received_s_perfect, received_thread_id) in receiver {
        received_count += 1;
        s_set.append(&mut received_s_set);
        for i in 0..NEAR_S_RANGE {
            s_perf[i].append(&mut received_s_perfect[i]);
        }

        // checkpoint
        println!{"after adding thread {received_thread_id}'s results, s_perfect is: {:?}", s_perf};

        // check to see if should stop spawning new threads
        if received_count == stop_thread {
            break;
        }

        // by now, we updated s_set, so we can start a new thread using those updates
        // as long as we still have numbers left to consider
        if thread_end <= max_size + 1 {
            let my_transmitter = transmitter.clone();
            let borrowed_s_set = s_set.clone();
            thread::spawn(move || {
                let (my_s_set, my_s_perfect, my_thread_id) = s_perfect(thread_start, thread_end, thread_id, borrowed_s_set);
                let my_additions = (my_s_set, my_s_perfect, my_thread_id);
                my_transmitter.send(my_additions).unwrap();
            });

            thread_start = thread_end;
            thread_end += THREAD_SIZE;
            thread_id += 1;
        }
    }

    // at this point, we know we're done w/ last thread; no need for handles

    // print out the table
    println!("Deficient (in S)");
    let mut label = S_DEFICIENT;
    for vec in s_perf {
       if label == 0 {
            println!("\nPerfect (in S)");
            println!("{label} --> {:?}\n", vec);
            println!("Abundant (not in S)");
       } else {
           println!("{label} --> {:?}", vec);
       }
       label+= 1;
    }
}

/* for the given range [start, end), find the numbers to be added to s_set
 * and the numbers to be added to s_perfect. then, return those additions
 * (as well as the thread id for debugging purposes)
 */
fn s_perfect(start:i32, end:i32, thread_id:i32, s_set:Vec<i32>) -> (Vec<i32>, [Vec<i32>; NEAR_S_RANGE + 1], i32) {
    // create two local vectors that will store the numbers to be added to s_set and s_perf from the given range
    let mut my_s_set:Vec<i32> = vec![];
    let mut my_s_perfect = [VEC; NEAR_S_RANGE + 1];
    
    // for every num in the given range, check to see if it is in the s set; if it is, then add it
    for n in start..end {
        let mut sum = 0;
     
        // check every num in the borrowed s_set
        for i in 0..s_set.len() {
            let s_num = s_set[i]; // read from borrowed s_set
            if s_num != 0 && n % s_num == 0 && s_num < n { // s_num is a factor of n and is not n itself
                sum += s_num;
            }
        }

        // now check every num in my s_set
        for i in 0..my_s_set.len() {
            let s_num = my_s_set[i]; // read from my s_set
            if n % s_num == 0 && s_num < n {
                sum += s_num;
            }
        }

        // check if n is in the range to be considered near s perfect
        let distance_from_s_perfect = sum - n;
        if distance_from_s_perfect >= S_DEFICIENT && distance_from_s_perfect <= S_ABUNDANT {
            my_s_perfect[(distance_from_s_perfect + S_ABUNDANT) as usize].push(n);
        }

        if sum <= n { // n is in the s set and needs to be added
            my_s_set.push(n);
        }
    }

    return (my_s_set, my_s_perfect, thread_id);
}