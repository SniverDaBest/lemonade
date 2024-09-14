use crate::randomness::Xorshift32;

pub fn quicksort<T: Ord>(arr: &mut [T]) {
    quicksort_helper(arr, 0, arr.len());
}

fn quicksort_helper<T: Ord>(arr: &mut [T], left: usize, right: usize) {
    if left < right {
        let pivot_index = partition(arr, left, right);
        quicksort_helper(arr, left, pivot_index);
        quicksort_helper(arr, pivot_index + 1, right);
    }
}

fn partition<T: Ord>(arr: &mut [T], left: usize, right: usize) -> usize {
    let pivot_index = right;  // Use the last element as the pivot
    let mut i = left;

    for j in left..right {
        if arr[j] <= arr[pivot_index] {
            arr.swap(i, j);
            i += 1;
        }
    }

    arr.swap(i, pivot_index);  // Swap pivot to its correct position
    i  // Return the pivot index
}

pub fn stupidsort<T: Ord + Copy>(arr: &mut [T], seed: u32) {
    let mut rng = Xorshift32::new(seed); // Initialize your Xorshift32 generator

    loop {
        let mut permuted = false;

        for i in 0..arr.len() {
            let j = rng.gen_range(i as u32, arr.len() as u32); // Generate a random index within the range
            arr.swap(i, j as usize); // Swap elements at positions i and j

            // Check if the array is sorted after every swap
            for k in 0..arr.len() - 1 {
                if arr[k] > arr[k + 1] {
                    permuted = true;
                    break;
                }
            }

            if !permuted {
                break;
            }
        }

        if !permuted {
            break;
        }
    }
}
